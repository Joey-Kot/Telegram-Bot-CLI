use std::env;
use std::fmt;

use url::Url;

use crate::error::{AppError, Result};

const DEFAULT_BASE_URL: &str = "https://api.telegram.org";

#[derive(Clone)]
pub struct BotToken(String);

impl BotToken {
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for BotToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BotToken(<redacted>)")
    }
}

impl fmt::Display for BotToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted>")
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub token: BotToken,
    pub base_url: Url,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let raw_token = env::var("TELEGRAM_BOT_TOKEN").map_err(|_| AppError::MissingBotToken)?;
        if raw_token.trim().is_empty() {
            return Err(AppError::EmptyBotToken);
        }

        let raw_base_url =
            env::var("TELEGRAM_BOT_API_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned());
        let base_url = Url::parse(raw_base_url.trim())
            .map_err(|_| AppError::InvalidBaseUrl("无法解析 URL".to_owned()))?;

        if !matches!(base_url.scheme(), "https" | "http") {
            return Err(AppError::InvalidBaseUrl(
                "只支持 http 或 https 协议".to_owned(),
            ));
        }
        if base_url.host().is_none() {
            return Err(AppError::InvalidBaseUrl("缺少主机名".to_owned()));
        }
        if base_url.query().is_some() || base_url.fragment().is_some() {
            return Err(AppError::InvalidBaseUrl(
                "不能包含查询参数或 fragment".to_owned(),
            ));
        }
        if !base_url.username().is_empty() || base_url.password().is_some() {
            return Err(AppError::InvalidBaseUrl("不能包含用户名或密码".to_owned()));
        }

        Ok(Self {
            token: BotToken(raw_token),
            base_url,
        })
    }
}
