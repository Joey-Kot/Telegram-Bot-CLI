use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("缺少必填环境变量 TELEGRAM_BOT_TOKEN")]
    MissingBotToken,

    #[error("环境变量 TELEGRAM_BOT_TOKEN 不能为空")]
    EmptyBotToken,

    #[error("TELEGRAM_BOT_API_BASE_URL 无效：{0}")]
    InvalidBaseUrl(String),

    #[error("{0}")]
    Input(String),

    #[error("无法读取 {label}：{path}")]
    InputRead { label: &'static str, path: PathBuf },

    #[error("{label} 必须是 UTF-8 文本：{path}")]
    InputNotUtf8 { label: &'static str, path: PathBuf },

    #[error("无法读取标准输入")]
    StdinRead,

    #[error("标准输入只能由一个 --*-file - 参数读取")]
    StdinAlreadyUsed,

    #[error("本地文件不存在、不可读取或不是普通文件：{path}")]
    InvalidLocalFile { path: PathBuf },

    #[error("无法打开本地文件：{path}")]
    FileOpen { path: PathBuf },

    #[error("无法读取本地文件元数据：{path}")]
    FileMetadata { path: PathBuf },

    #[error("无法连接到 Telegram Bot API")]
    Network,

    #[error("无法读取 Telegram Bot API 响应")]
    ResponseRead,

    #[error("Telegram Bot API 返回的内容不是有效 JSON（HTTP {status}）")]
    InvalidApiResponse { status: u16 },

    #[error("无法构造 multipart 请求")]
    Multipart,

    #[error("无法写入标准输出")]
    OutputWrite,
}

impl AppError {
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::MissingBotToken
            | Self::EmptyBotToken
            | Self::InvalidBaseUrl(_)
            | Self::Input(_)
            | Self::InputRead { .. }
            | Self::InputNotUtf8 { .. }
            | Self::StdinRead
            | Self::StdinAlreadyUsed
            | Self::InvalidLocalFile { .. } => 2,
            Self::FileOpen { .. }
            | Self::FileMetadata { .. }
            | Self::Network
            | Self::ResponseRead
            | Self::InvalidApiResponse { .. }
            | Self::Multipart
            | Self::OutputWrite => 1,
        }
    }
}
