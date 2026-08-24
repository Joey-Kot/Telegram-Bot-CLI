use std::path::Path;
use std::time::Duration;

use reqwest::multipart::{Form, Part};
use reqwest::{Client, Proxy};
use serde::Deserialize;
use serde_json::Value;
use tokio_util::io::ReaderStream;
use url::Url;

use crate::config::Config;
use crate::error::{AppError, Result};

use super::socks4::{Socks4Config, Socks4Relay};
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
    _socks4_relay: Option<Socks4Relay>,
}

#[derive(Debug, Deserialize)]
struct ResponseEnvelope {
    ok: bool,
}

impl TelegramClient {
    pub async fn new(config: Config, proxy_url: Option<&str>) -> Result<Self> {
        let configured_proxy = match proxy_url {
            Some(proxy_url) => Some(build_proxy(proxy_url).await?),
            None => None,
        };

        let mut builder = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .user_agent(concat!("tgpush/", env!("CARGO_PKG_VERSION")));
        if let Some(configured_proxy) = &configured_proxy {
            builder = builder.proxy(configured_proxy.proxy.clone());
        }
        let http = builder.build().map_err(|_| AppError::Network)?;

        Ok(Self {
            http,
            config,
            _socks4_relay: configured_proxy.and_then(|proxy| proxy.socks4_relay),
        })
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
    socks4_relay: Option<Socks4Relay>,
}

async fn build_proxy(raw_proxy_url: &str) -> Result<ConfiguredProxy> {
    let raw_proxy_url = raw_proxy_url.trim();
    if raw_proxy_url.is_empty() {
        return Err(AppError::InvalidProxy("不能为空".to_owned()));
    }

    let mut proxy_url =
        Url::parse(raw_proxy_url).map_err(|_| AppError::InvalidProxy("无法解析 URL".to_owned()))?;
    if proxy_url.host().is_none() {
        return Err(AppError::InvalidProxy("缺少主机名".to_owned()));
    }
    if proxy_url.query().is_some() || proxy_url.fragment().is_some() {
        return Err(AppError::InvalidProxy(
            "不能包含查询参数或 fragment".to_owned(),
        ));
    }

    if proxy_url.scheme() == "socks" {
        proxy_url
            .set_scheme("socks5")
            .map_err(|_| AppError::InvalidProxy("不支持的协议".to_owned()))?;
    }

    match proxy_url.scheme() {
        "http" | "https" | "socks5" | "socks5h" => {}
        "socks4" | "socks4a" => {
            let socks4_config = Socks4Config::from_url(&proxy_url)?;
            let (socks4_relay, local_proxy_url) = Socks4Relay::start(socks4_config).await?;
            let proxy = Proxy::all(local_proxy_url)
                .map_err(|_| AppError::InvalidProxy("无法构造代理配置".to_owned()))?;
            return Ok(ConfiguredProxy {
                proxy,
                socks4_relay: Some(socks4_relay),
            });
        }
        _ => {
            return Err(AppError::InvalidProxy(
                "只支持 http、https、socks4、socks4a、socks5、socks5h 或 socks 协议".to_owned(),
            ));
        }
    }

    let proxy =
        Proxy::all(proxy_url).map_err(|_| AppError::InvalidProxy("无法构造代理配置".to_owned()))?;
    Ok(ConfiguredProxy {
        proxy,
        socks4_relay: None,
    })
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
