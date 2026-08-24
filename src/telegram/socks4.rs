use std::io;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use percent_encoding::percent_decode_str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, lookup_host};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use url::Url;

use crate::error::{AppError, Result};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_HTTP_PROXY_HEADERS: usize = 64 * 1024;
const BAD_GATEWAY_RESPONSE: &[u8] =
    b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
const CONNECTION_ESTABLISHED_RESPONSE: &[u8] = b"HTTP/1.1 200 Connection Established\r\n\r\n";

#[derive(Clone)]
pub(crate) struct Socks4Config {
    proxy_host: String,
    proxy_port: u16,
    user_id: Vec<u8>,
    remote_dns: bool,
}

impl Socks4Config {
    pub(crate) fn from_url(url: &Url) -> Result<Self> {
        if url.password().is_some() {
            return Err(AppError::InvalidProxy(
                "SOCKS4 不支持密码认证；请使用 SOCKS5".to_owned(),
            ));
        }

        let user_id = percent_decode_str(url.username())
            .decode_utf8()
            .map_err(|_| AppError::InvalidProxy("用户名不是有效 UTF-8".to_owned()))?
            .into_owned()
            .into_bytes();
        if user_id.contains(&0) {
            return Err(AppError::InvalidProxy(
                "SOCKS4 用户名不能包含空字节".to_owned(),
            ));
        }

        let proxy_host = url
            .host_str()
            .ok_or_else(|| AppError::InvalidProxy("缺少主机名".to_owned()))?
            .to_owned();
        let proxy_port = url.port().unwrap_or(1080);

        Ok(Self {
            proxy_host,
            proxy_port,
            user_id,
            remote_dns: url.scheme() == "socks4a",
        })
    }

    async fn connect(&self, target: Target) -> io::Result<TcpStream> {
        timeout(CONNECT_TIMEOUT, self.connect_inner(target))
            .await
            .map_err(|_| {
                io::Error::new(io::ErrorKind::TimedOut, "SOCKS4 proxy connection timed out")
            })?
    }

    async fn connect_inner(&self, target: Target) -> io::Result<TcpStream> {
        let destination = if self.remote_dns {
            Socks4Destination::Domain(target)
        } else {
            Socks4Destination::Ip {
                address: resolve_ipv4(&target).await?,
                port: target.port,
            }
        };
        let mut stream = TcpStream::connect((self.proxy_host.as_str(), self.proxy_port)).await?;
        let request = self.request_bytes(destination);
        stream.write_all(&request).await?;

        let mut response = [0; 8];
        stream.read_exact(&mut response).await?;
        if response[0] != 0 || response[1] != 90 {
            return Err(io::Error::other("SOCKS4 proxy rejected the connection"));
        }

        Ok(stream)
    }

    fn request_bytes(&self, destination: Socks4Destination) -> Vec<u8> {
        let mut request = Vec::with_capacity(9 + self.user_id.len() + 256);
        request.extend_from_slice(&[4, 1]);
        request.extend_from_slice(&destination.port().to_be_bytes());
        match &destination {
            Socks4Destination::Ip { address, .. } => request.extend_from_slice(&address.octets()),
            Socks4Destination::Domain(_) => request.extend_from_slice(&[0, 0, 0, 1]),
        }
        request.extend_from_slice(&self.user_id);
        request.push(0);
        if let Socks4Destination::Domain(target) = destination {
            request.extend_from_slice(target.host.as_bytes());
            request.push(0);
        }
        request
    }
}

pub(crate) struct Socks4Relay {
    task: JoinHandle<()>,
}

impl Socks4Relay {
    pub(crate) async fn start(config: Socks4Config) -> Result<(Self, Url)> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|_| AppError::Network)?;
        let address = listener.local_addr().map_err(|_| AppError::Network)?;
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                serve_connection(stream, &config).await;
            }
        });
        let local_proxy_url =
            Url::parse(&format!("http://{address}")).map_err(|_| AppError::Network)?;

        Ok((Self { task }, local_proxy_url))
    }
}

impl Drop for Socks4Relay {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[derive(Clone)]
struct Target {
    host: String,
    port: u16,
}

enum Socks4Destination {
    Ip { address: Ipv4Addr, port: u16 },
    Domain(Target),
}

impl Socks4Destination {
    fn port(&self) -> u16 {
        match self {
            Self::Ip { port, .. } => *port,
            Self::Domain(target) => target.port,
        }
    }
}

enum ProxyRequest {
    Connect { target: Target, pending: Vec<u8> },
    Forward { target: Target, request: Vec<u8> },
}

async fn serve_connection(mut downstream: TcpStream, config: &Socks4Config) {
    let request = match read_request(&mut downstream).await.and_then(parse_request) {
        Ok(request) => request,
        Err(_) => {
            let _ = downstream.write_all(BAD_GATEWAY_RESPONSE).await;
            return;
        }
    };

    let target = match &request {
        ProxyRequest::Connect { target, .. } | ProxyRequest::Forward { target, .. } => {
            target.clone()
        }
    };
    let mut upstream = match config.connect(target).await {
        Ok(stream) => stream,
        Err(_) => {
            let _ = downstream.write_all(BAD_GATEWAY_RESPONSE).await;
            return;
        }
    };

    match request {
        ProxyRequest::Connect { pending, .. } => {
            if downstream
                .write_all(CONNECTION_ESTABLISHED_RESPONSE)
                .await
                .is_err()
            {
                return;
            }
            if !pending.is_empty() && upstream.write_all(&pending).await.is_err() {
                return;
            }
        }
        ProxyRequest::Forward { request, .. } => {
            if upstream.write_all(&request).await.is_err() {
                return;
            }
        }
    }

    let _ = tokio::io::copy_bidirectional(&mut downstream, &mut upstream).await;
}

async fn read_request(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut request = Vec::new();
    let mut buffer = [0; 4096];

    loop {
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "HTTP proxy client closed the connection before sending headers",
            ));
        }
        request.extend_from_slice(&buffer[..bytes_read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            return Ok(request);
        }
        if request.len() > MAX_HTTP_PROXY_HEADERS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "HTTP proxy request headers are too large",
            ));
        }
    }
}

fn parse_request(request: Vec<u8>) -> io::Result<ProxyRequest> {
    let header_end = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
        .ok_or_else(|| invalid_request("missing HTTP header terminator"))?;
    let first_line_end = request
        .windows(2)
        .position(|window| window == b"\r\n")
        .ok_or_else(|| invalid_request("missing HTTP request line"))?;
    let first_line = std::str::from_utf8(&request[..first_line_end])
        .map_err(|_| invalid_request("HTTP request line is not UTF-8"))?;
    let mut parts = first_line.splitn(3, ' ');
    let method = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid_request("missing HTTP method"))?;
    let target = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid_request("missing HTTP request target"))?;
    let version = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid_request("missing HTTP version"))?;

    if method.eq_ignore_ascii_case("CONNECT") {
        return Ok(ProxyRequest::Connect {
            target: parse_connect_target(target)?,
            pending: request[header_end..].to_vec(),
        });
    }

    let url = Url::parse(target).map_err(|_| invalid_request("invalid HTTP proxy target"))?;
    if url.scheme() != "http" {
        return Err(invalid_request("unsupported HTTP proxy target scheme"));
    }
    let target = target_from_url(&url)?;
    let origin_form = origin_form(&url);
    let mut rewritten = format!("{method} {origin_form} {version}\r\n").into_bytes();
    rewritten.extend_from_slice(&request[first_line_end + 2..]);

    Ok(ProxyRequest::Forward {
        target,
        request: rewritten,
    })
}

fn parse_connect_target(authority: &str) -> io::Result<Target> {
    let url = Url::parse(&format!("http://{authority}"))
        .map_err(|_| invalid_request("invalid CONNECT target"))?;
    if !url.username().is_empty() || url.password().is_some() || url.path() != "/" {
        return Err(invalid_request("invalid CONNECT target"));
    }
    target_from_url(&url)
}

fn target_from_url(url: &Url) -> io::Result<Target> {
    let host = url
        .host_str()
        .ok_or_else(|| invalid_request("target host is missing"))?
        .to_owned();
    let port = url
        .port_or_known_default()
        .ok_or_else(|| invalid_request("target port is missing"))?;
    Ok(Target { host, port })
}

fn origin_form(url: &Url) -> String {
    let mut origin_form = url.path().to_owned();
    if origin_form.is_empty() {
        origin_form.push('/');
    }
    if let Some(query) = url.query() {
        origin_form.push('?');
        origin_form.push_str(query);
    }
    origin_form
}

async fn resolve_ipv4(target: &Target) -> io::Result<Ipv4Addr> {
    if let Ok(IpAddr::V4(address)) = target.host.parse() {
        return Ok(address);
    }
    if target.host.parse::<IpAddr>().is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "SOCKS4 does not support IPv6 targets",
        ));
    }

    lookup_host((target.host.as_str(), target.port))
        .await?
        .find_map(|address| match address.ip() {
            IpAddr::V4(address) => Some(address),
            IpAddr::V6(_) => None,
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "target host has no IPv4 address for SOCKS4",
            )
        })
}

fn invalid_request(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
