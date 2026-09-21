use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Proxy};
use serde::Deserialize;
use serde_json::Value;
use tokio_util::io::ReaderStream;
use url::Url;

use crate::config::Config;
use crate::error::{AppError, Result};

use super::{FilePart, RequestSpec};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug)]
pub struct ApiResponse {
    pub raw_body: Vec<u8>,
    pub ok: bool,
}

pub struct TelegramClient {
    http: Client,
    config: Config,
}

#[derive(Debug, Deserialize)]
struct ResponseEnvelope {
    ok: bool,
}

impl TelegramClient {
    pub fn new(config: Config, proxy_url: Option<&str>) -> Result<Self> {
        let mut builder = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .user_agent(concat!("tgpush/", env!("CARGO_PKG_VERSION")));
        if let Some(proxy_url) = proxy_url {
            let configured_proxy = build_proxy(proxy_url)?;
            if configured_proxy.prefer_ipv4_dns_results {
                let resolver: Arc<dyn Resolve> = Arc::new(Ipv4FirstResolver);
                builder = builder.dns_resolver(resolver);
            }
            builder = builder.proxy(configured_proxy.proxy);
        }
        let http = builder.build().map_err(|_| AppError::Network)?;

        Ok(Self { http, config })
    }

    pub async fn execute(&self, spec: RequestSpec) -> Result<ApiResponse> {
        let endpoint = self.endpoint(spec.method)?;
        let request = if spec.files.is_empty() {
            let request = self.http.post(endpoint);
            if spec.fields.is_empty() {
                request
            } else {
                request.json(&Value::Object(spec.fields))
            }
        } else {
            let form = build_form(spec.fields, spec.files).await?;
            self.http.post(endpoint).multipart(form)
        };

        let response = request.send().await.map_err(|_| AppError::Network)?;
        let status = response.status();
        let raw_body = response
            .bytes()
            .await
            .map_err(|_| AppError::ResponseRead)?
            .to_vec();
        let envelope = serde_json::from_slice::<ResponseEnvelope>(&raw_body).map_err(|_| {
            AppError::InvalidApiResponse {
                status: status.as_u16(),
            }
        })?;

        Ok(ApiResponse {
            raw_body,
            ok: status.is_success() && envelope.ok,
        })
    }

    fn endpoint(&self, method: &str) -> Result<Url> {
        let mut base = self
            .config
            .base_url
            .as_str()
            .trim_end_matches('/')
            .to_owned();
        base.push_str("/bot");
        base.push_str(self.config.token.expose());
        base.push('/');
        base.push_str(method);

        Url::parse(&base).map_err(|_| AppError::Network)
    }
}

struct ConfiguredProxy {
    proxy: Proxy,
    prefer_ipv4_dns_results: bool,
}

fn build_proxy(raw_proxy_url: &str) -> Result<ConfiguredProxy> {
    let proxy_url =
        Url::parse(raw_proxy_url).map_err(|_| AppError::InvalidProxy("无法解析 URL".to_owned()))?;
    if proxy_url.host().is_none() {
        return Err(AppError::InvalidProxy("缺少主机名".to_owned()));
    }

    match proxy_url.scheme() {
        "http" | "https" | "socks5" | "socks5h" => {}
        "socks4" | "socks4a" => {
            if !proxy_url.username().is_empty() || proxy_url.password().is_some() {
                return Err(AppError::InvalidProxy(
                    "SOCKS4 和 SOCKS4a 不支持 URL 认证；请使用 SOCKS5".to_owned(),
                ));
            }
        }
        _ => {
            return Err(AppError::InvalidProxy(
                "只支持 HTTP、HTTPS、SOCKS4、SOCKS4a、SOCKS5 或 SOCKS5h URL".to_owned(),
            ));
        }
    }

    let prefer_ipv4_dns_results = proxy_url.scheme() == "socks4";
    let proxy = Proxy::all(proxy_url)
        .map_err(|_| AppError::InvalidProxy("无法构造 reqwest 代理配置".to_owned()))?;

    Ok(ConfiguredProxy {
        proxy,
        prefer_ipv4_dns_results,
    })
}

struct Ipv4FirstResolver;

impl Resolve for Ipv4FirstResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_owned();
        Box::pin(async move {
            let mut addresses = tokio::net::lookup_host((host.as_str(), 0))
                .await?
                .collect::<Vec<_>>();
            prefer_ipv4(&mut addresses);
            Ok(Box::new(addresses.into_iter()) as Addrs)
        })
    }
}

fn prefer_ipv4(addresses: &mut [SocketAddr]) {
    addresses.sort_by_key(|address| address.is_ipv6());
}

async fn build_form(fields: serde_json::Map<String, Value>, files: Vec<FilePart>) -> Result<Form> {
    let mut form = Form::new();

    for (name, value) in fields {
        form = form.text(name, form_value(value));
    }

    for file_part in files {
        form = form.part(
            file_part.field_name,
            build_file_part(&file_part.path).await?,
        );
    }

    Ok(form)
}

fn form_value(value: Value) -> String {
    match value {
        Value::String(value) => value,
        other => other.to_string(),
    }
}

async fn build_file_part(path: &Path) -> Result<Part> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|_| AppError::FileMetadata {
            path: path.to_path_buf(),
        })?;
    if !metadata.is_file() {
        return Err(AppError::InvalidLocalFile {
            path: path.to_path_buf(),
        });
    }

    let file = tokio::fs::File::open(path)
        .await
        .map_err(|_| AppError::FileOpen {
            path: path.to_path_buf(),
        })?;
    let stream = ReaderStream::new(file);
    let body = reqwest::Body::wrap_stream(stream);
    let filename = path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "upload".to_owned());
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    Part::stream(body)
        .file_name(filename)
        .mime_str(mime.as_ref())
        .map_err(|_| AppError::Multipart)
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

    use super::{build_proxy, prefer_ipv4};

    #[test]
    fn socks4_prefers_ipv4_dns_results() {
        let proxy = build_proxy("socks4://127.0.0.1:1080").expect("build SOCKS4 proxy");
        assert!(proxy.prefer_ipv4_dns_results);

        let mut addresses = vec![
            SocketAddr::from((Ipv6Addr::LOCALHOST, 443)),
            SocketAddr::from((Ipv4Addr::LOCALHOST, 443)),
        ];
        prefer_ipv4(&mut addresses);

        assert!(addresses[0].is_ipv4());
        assert!(addresses[1].is_ipv6());
    }
}
