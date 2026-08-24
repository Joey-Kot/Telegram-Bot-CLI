use std::path::Path;
use std::time::Duration;

use reqwest::Client;
use reqwest::multipart::{Form, Part};
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

#[derive(Clone)]
pub struct TelegramClient {
    http: Client,
    config: Config,
}

#[derive(Debug, Deserialize)]
struct ResponseEnvelope {
    ok: bool,
}

impl TelegramClient {
    pub fn new(config: Config) -> Result<Self> {
        let http = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .user_agent(concat!("tgpush/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| AppError::Network)?;

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
