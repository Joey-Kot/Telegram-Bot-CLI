mod client;
mod multipart;

pub use client::{ApiResponse, TelegramClient};
pub use multipart::{FilePart, RequestSpec};
