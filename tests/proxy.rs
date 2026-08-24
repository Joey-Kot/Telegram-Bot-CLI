use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

type ProxyResult = std::result::Result<(), String>;

fn tgpush() -> Command {
    let mut command = Command::cargo_bin("tgpush").expect("tgpush binary should be available");
    command.env("TELEGRAM_BOT_TOKEN", "test-token");
    command
}

#[tokio::test]
async fn http_proxy_uses_url_credentials() {
    let proxy = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/getMe"))
        .and(header(
            "proxy-authorization",
            "Basic cHJveHktdXNlcjpwcm94eS1wYXNzd29yZA==",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(b"{\"ok\":true,\"result\":{}}", "application/json"),
        )
        .expect(1)
        .mount(&proxy)
        .await;

    let proxy_url = format!("http://proxy-user:proxy-password@{}", proxy.address());
    let output = tgpush()
        .args(["check", "--proxy", &proxy_url])
        .env("TELEGRAM_BOT_API_BASE_URL", "http://telegram.example")
        .output()
        .expect("run tgpush");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"{\"ok\":true,\"result\":{}}");
    assert!(output.stderr.is_empty());
}

#[test]
fn socks4_proxy_uses_url_user_id() {
    let (port, result) = start_proxy(|stream| {
        let mut request = [0; 8];
        stream
            .read_exact(&mut request)
            .map_err(|error| error.to_string())?;
        if request != [4, 1, 0x30, 0x39, 127, 0, 0, 1] {
            return Err(format!("unexpected SOCKS4 request: {request:?}"));
        }

        let mut user_id = [0; 11];
        stream
            .read_exact(&mut user_id)
            .map_err(|error| error.to_string())?;
        if user_id != *b"proxy-user\0" {
            return Err(format!("unexpected SOCKS4 user ID: {user_id:?}"));
        }

        stream
            .write_all(&[0, 90, 0x30, 0x39, 127, 0, 0, 1])
            .map_err(|error| error.to_string())?;
        assert_get_me_request(stream)?;
        write_success_response(stream)
    });

    let proxy_url = format!("socks4://proxy-user@127.0.0.1:{port}");
    let output = tgpush()
        .args(["--proxy", &proxy_url, "check"])
        .env("TELEGRAM_BOT_API_BASE_URL", "http://127.0.0.1:12345")
        .output()
        .expect("run tgpush");

    assert_proxy_completed(result);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"{\"ok\":true,\"result\":{}}");
    assert!(output.stderr.is_empty());
}

#[test]
fn socks4_proxy_tunnels_https_requests() {
    let (port, result) = start_proxy(|stream| {
        let mut request = [0; 8];
        stream
            .read_exact(&mut request)
            .map_err(|error| error.to_string())?;
        if request != [4, 1, 0x30, 0x39, 127, 0, 0, 1] {
            return Err(format!("unexpected SOCKS4 request: {request:?}"));
        }

        let mut user_id_end = [0; 1];
        stream
            .read_exact(&mut user_id_end)
            .map_err(|error| error.to_string())?;
        if user_id_end != [0] {
            return Err("unexpected SOCKS4 user ID".to_owned());
        }

        stream
            .write_all(&[0, 90, 0x30, 0x39, 127, 0, 0, 1])
            .map_err(|error| error.to_string())?;
        let mut tls_record_type = [0; 1];
        stream
            .read_exact(&mut tls_record_type)
            .map_err(|error| error.to_string())?;
        if tls_record_type != [22] {
            return Err(format!(
                "expected a TLS handshake record, got {tls_record_type:?}"
            ));
        }

        Ok(())
    });

    let proxy_url = format!("socks4://127.0.0.1:{port}");
    let output = tgpush()
        .args(["check", "--proxy", &proxy_url])
        .env("TELEGRAM_BOT_API_BASE_URL", "https://127.0.0.1:12345")
        .output()
        .expect("run tgpush");

    assert_proxy_completed(result);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("无法连接到 Telegram Bot API"));
}

#[test]
fn socks5_proxy_uses_url_credentials() {
    let (port, result) = start_proxy(|stream| {
        let credentials = read_socks5_credentials(stream)?;
        if credentials != ("proxy-user".to_owned(), "proxy@password".to_owned()) {
            return Err(format!("unexpected SOCKS5 credentials: {credentials:?}"));
        }

        let destination = read_socks5_destination(stream)?;
        if destination != ("127.0.0.1".to_owned(), 12345) {
            return Err(format!("unexpected SOCKS5 destination: {destination:?}"));
        }

        stream
            .write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0])
            .map_err(|error| error.to_string())?;
        assert_get_me_request(stream)?;
        write_success_response(stream)
    });

    let proxy_url = format!("socks5://proxy-user:proxy%40password@127.0.0.1:{port}");
    let output = tgpush()
        .args(["check", "--proxy", &proxy_url])
        .env("TELEGRAM_BOT_API_BASE_URL", "http://127.0.0.1:12345")
        .output()
        .expect("run tgpush");

    assert_proxy_completed(result);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"{\"ok\":true,\"result\":{}}");
    assert!(output.stderr.is_empty());
}

#[test]
fn invalid_proxy_values_are_rejected_without_exposing_credentials() {
    tgpush()
        .args(["check", "--proxy", "ftp://proxy-user:secret@127.0.0.1:21"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("代理 URL 无效：只支持"))
        .stderr(predicate::str::contains("secret").not());

    tgpush()
        .args([
            "check",
            "--proxy",
            "socks4://proxy-user:secret@127.0.0.1:1080",
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("SOCKS4 不支持密码认证"))
        .stderr(predicate::str::contains("secret").not());
}

fn start_proxy<F>(handler: F) -> (u16, Receiver<ProxyResult>)
where
    F: FnOnce(&mut TcpStream) -> ProxyResult + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind proxy listener");
    let port = listener.local_addr().expect("read proxy address").port();
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let result = (|| -> ProxyResult {
            let (mut stream, _) = listener.accept().map_err(|error| error.to_string())?;
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .map_err(|error| error.to_string())?;
            stream
                .set_write_timeout(Some(Duration::from_secs(5)))
                .map_err(|error| error.to_string())?;
            handler(&mut stream)
        })();
        let _ = sender.send(result);
    });

    (port, receiver)
}

fn read_socks5_credentials(stream: &mut TcpStream) -> ProxyResultWith<(String, String)> {
    let mut greeting = [0; 3];
    stream
        .read_exact(&mut greeting)
        .map_err(|error| error.to_string())?;
    if greeting != [5, 1, 2] {
        return Err(format!("unexpected SOCKS5 greeting: {greeting:?}"));
    }
    stream
        .write_all(&[5, 2])
        .map_err(|error| error.to_string())?;

    let mut authentication = [0; 2];
    stream
        .read_exact(&mut authentication)
        .map_err(|error| error.to_string())?;
    if authentication[0] != 1 {
        return Err(format!(
            "unexpected SOCKS5 authentication version: {}",
            authentication[0]
        ));
    }

    let mut username = vec![0; authentication[1] as usize];
    stream
        .read_exact(&mut username)
        .map_err(|error| error.to_string())?;
    let mut password_length = [0; 1];
    stream
        .read_exact(&mut password_length)
        .map_err(|error| error.to_string())?;
    let mut password = vec![0; password_length[0] as usize];
    stream
        .read_exact(&mut password)
        .map_err(|error| error.to_string())?;
    stream
        .write_all(&[1, 0])
        .map_err(|error| error.to_string())?;

    Ok((
        String::from_utf8(username).map_err(|error| error.to_string())?,
        String::from_utf8(password).map_err(|error| error.to_string())?,
    ))
}

fn read_socks5_destination(stream: &mut TcpStream) -> ProxyResultWith<(String, u16)> {
    let mut request = [0; 4];
    stream
        .read_exact(&mut request)
        .map_err(|error| error.to_string())?;
    if request[..3] != [5, 1, 0] {
        return Err(format!("unexpected SOCKS5 request header: {request:?}"));
    }

    let host = match request[3] {
        1 => {
            let mut address = [0; 4];
            stream
                .read_exact(&mut address)
                .map_err(|error| error.to_string())?;
            address
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(".")
        }
        3 => {
            let mut length = [0; 1];
            stream
                .read_exact(&mut length)
                .map_err(|error| error.to_string())?;
            let mut address = vec![0; length[0] as usize];
            stream
                .read_exact(&mut address)
                .map_err(|error| error.to_string())?;
            String::from_utf8(address).map_err(|error| error.to_string())?
        }
        address_type => return Err(format!("unexpected SOCKS5 address type: {address_type}")),
    };
    let mut port = [0; 2];
    stream
        .read_exact(&mut port)
        .map_err(|error| error.to_string())?;

    Ok((host, u16::from_be_bytes(port)))
}

fn assert_get_me_request(stream: &mut TcpStream) -> ProxyResult {
    let request = read_http_headers(stream)?;
    if !request.starts_with("POST /bottest-token/getMe HTTP/1.1\r\n") {
        return Err(format!("unexpected HTTP request: {request:?}"));
    }
    Ok(())
}

fn read_http_headers(stream: &mut TcpStream) -> ProxyResultWith<String> {
    let mut request = Vec::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if bytes_read == 0 {
            return Err("connection closed before the HTTP headers arrived".to_owned());
        }
        request.extend_from_slice(&buffer[..bytes_read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            return String::from_utf8(request).map_err(|error| error.to_string());
        }
        if request.len() > 16 * 1024 {
            return Err("HTTP headers are unexpectedly large".to_owned());
        }
    }
}

fn write_success_response(stream: &mut TcpStream) -> ProxyResult {
    let body = b"{\"ok\":true,\"result\":{}}";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        std::str::from_utf8(body).expect("success response is valid UTF-8")
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| error.to_string())
}

fn assert_proxy_completed(result: Receiver<ProxyResult>) {
    result
        .recv_timeout(Duration::from_secs(5))
        .expect("proxy server should finish")
        .expect("proxy server should receive the expected protocol exchange");
}

type ProxyResultWith<T> = std::result::Result<T, String>;
