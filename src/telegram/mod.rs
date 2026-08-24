mod client;
mod multipart;
mod socks4;

pub use client::{ApiResponse, TelegramClient};
pub use multipart::{FilePart, RequestSpec};
