use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "tgpush",
    about = "Telegram Bot API 的命令行客户端",
    version,
    after_help = "环境变量：\n  TELEGRAM_BOT_TOKEN  必填。格式为 <Bot 数字 ID>:<密钥>，例如\n                      123456789:AAExampleBotToken_ReplaceMe1234567890\n  TELEGRAM_BOT_API_BASE_URL  可选，默认 https://api.telegram.org\n\n最小示例：\n  export TELEGRAM_BOT_TOKEN='123456789:AAExampleBotToken_ReplaceMe1234567890'\n  tgpush check\n\n示例 Token 仅用于说明格式，不能用于认证。",
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(long, global = true, action = ArgAction::Help, help = "显示帮助")]
    #[allow(dead_code)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version, help = "显示版本")]
    #[allow(dead_code)]
    version: Option<bool>,

    /// 代理 URL；支持 HTTP、HTTPS、SOCKS4 和 SOCKS5
    #[arg(long, global = true, value_name = "URL")]
    pub proxy: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// 调用 getMe 检查 Bot API 连通性和 Token 有效性
    Check,
    /// 发送文本消息
    Message(MessageArgs),
    /// 转发单条消息
    ForwardMessage(ForwardMessageArgs),
    /// 转发多条消息
    ForwardMessages(ForwardMessagesArgs),
    /// 发送图片
    Photo(PhotoArgs),
    /// 发送音频
    Audio(AudioArgs),
    /// 发送文档
    Document(DocumentArgs),
    /// 发送视频
    Video(VideoArgs),
    /// 发送动画（GIF 或无声 MPEG-4）
    Animation(AnimationArgs),
    /// 发送语音消息
    Voice(VoiceArgs),
}

#[derive(Debug, Clone, Args)]
pub struct TargetArgs {
    /// 目标聊天 ID 或 @username
    #[arg(long, allow_negative_numbers = true)]
    pub chat_id: String,

    /// Forum 话题 ID
    #[arg(long)]
    pub message_thread_id: Option<i64>,

    /// Direct Messages 话题 ID
    #[arg(long)]
    pub direct_messages_topic_id: Option<i64>,
}

#[derive(Debug, Clone, Args)]
pub struct SendContextArgs {
    /// Business 连接 ID
    #[arg(long)]
    pub business_connection_id: Option<String>,

    /// 临时消息的接收用户 ID
    #[arg(long)]
    pub receiver_user_id: Option<i64>,

    /// 触发临时消息的 Callback Query ID
    #[arg(long)]
    pub callback_query_id: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct SendOptionsArgs {
    /// 静默发送
    #[arg(long)]
    pub disable_notification: bool,

    /// 保护消息内容，禁止转发和保存
    #[arg(long)]
    pub protect_content: bool,

    /// 允许付费广播；会消耗 Telegram Stars
    #[arg(long)]
    pub allow_paid_broadcast: bool,

    /// 私聊消息特效 ID
    #[arg(long)]
    pub message_effect_id: Option<String>,

    /// SuggestedPostParameters JSON 对象
    #[arg(long)]
    pub suggested_post_parameters_json: Option<String>,

    /// SuggestedPostParameters JSON 文件；使用 - 从标准输入读取
    #[arg(long)]
    pub suggested_post_parameters_json_file: Option<PathBuf>,

    /// ReplyParameters JSON 对象
    #[arg(long)]
    pub reply_parameters_json: Option<String>,

    /// ReplyParameters JSON 文件；使用 - 从标准输入读取
    #[arg(long)]
    pub reply_parameters_json_file: Option<PathBuf>,

    /// ReplyMarkup JSON 对象
    #[arg(long)]
    pub reply_markup_json: Option<String>,

    /// ReplyMarkup JSON 文件；使用 - 从标准输入读取
    #[arg(long)]
    pub reply_markup_json_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct CaptionArgs {
    /// 媒体标题文本
    #[arg(long)]
    pub caption: Option<String>,

    /// 媒体标题 UTF-8 文件；使用 - 从标准输入读取
    #[arg(long)]
    pub caption_file: Option<PathBuf>,

    /// 标题解析模式：MarkdownV2、HTML 或 Markdown；自动处理对应格式的转义
    #[arg(long)]
    pub parse_mode: Option<String>,

    /// MessageEntity JSON 数组
    #[arg(long)]
    pub caption_entities_json: Option<String>,

    /// MessageEntity JSON 文件；使用 - 从标准输入读取
    #[arg(long)]
    pub caption_entities_json_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct MediaBaseArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    #[command(flatten)]
    pub context: SendContextArgs,

    #[command(flatten)]
    pub send_options: SendOptionsArgs,

    #[command(flatten)]
    pub caption: CaptionArgs,
}

#[derive(Debug, Clone, Args)]
pub struct MessageArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    #[command(flatten)]
    pub context: SendContextArgs,

    #[command(flatten)]
    pub send_options: SendOptionsArgs,

    /// 要发送的文本；不会解释反斜杠转义
    #[arg(long)]
    pub text: Option<String>,

    /// UTF-8 文本文件；使用 - 从标准输入读取
    #[arg(long)]
    pub text_file: Option<PathBuf>,

    /// 文本解析模式：MarkdownV2、HTML 或 Markdown；自动处理对应格式的转义
    #[arg(long)]
    pub parse_mode: Option<String>,

    /// MessageEntity JSON 数组
    #[arg(long)]
    pub entities_json: Option<String>,

    /// MessageEntity JSON 文件；使用 - 从标准输入读取
    #[arg(long)]
    pub entities_json_file: Option<PathBuf>,

    /// LinkPreviewOptions JSON 对象
    #[arg(long)]
    pub link_preview_options_json: Option<String>,

    /// LinkPreviewOptions JSON 文件；使用 - 从标准输入读取
    #[arg(long)]
    pub link_preview_options_json_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct ForwardMessageArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    /// 原消息所在聊天 ID 或 @username
    #[arg(long, allow_negative_numbers = true)]
    pub from_chat_id: String,

    /// 要转发的消息 ID
    #[arg(long)]
    pub message_id: i64,

    /// 转发视频的新起始时间戳（秒）
    #[arg(long)]
    pub video_start_timestamp: Option<u32>,

    /// 静默发送
    #[arg(long)]
    pub disable_notification: bool,

    /// 保护消息内容，禁止转发和保存
    #[arg(long)]
    pub protect_content: bool,

    /// 私聊消息特效 ID
    #[arg(long)]
    pub message_effect_id: Option<String>,

    /// SuggestedPostParameters JSON 对象
    #[arg(long)]
    pub suggested_post_parameters_json: Option<String>,

    /// SuggestedPostParameters JSON 文件；使用 - 从标准输入读取
    #[arg(long)]
    pub suggested_post_parameters_json_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct ForwardMessagesArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    /// 原消息所在聊天 ID 或 @username
    #[arg(long, allow_negative_numbers = true)]
    pub from_chat_id: String,

    /// 要转发的消息 ID；重复使用，必须严格递增
    #[arg(long, required = true)]
    pub message_id: Vec<i64>,

    /// 静默发送
    #[arg(long)]
    pub disable_notification: bool,

    /// 保护消息内容，禁止转发和保存
    #[arg(long)]
    pub protect_content: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PhotoArgs {
    #[command(flatten)]
    pub base: MediaBaseArgs,

    /// 图片的 HTTP URL 或 Telegram file_id
    #[arg(long)]
    pub photo: Option<String>,

    /// 要上传的本地图片路径
    #[arg(long)]
    pub photo_file: Option<PathBuf>,

    /// 将标题显示在媒体上方
    #[arg(long)]
    pub show_caption_above_media: bool,

    /// 对图片添加剧透遮罩
    #[arg(long)]
    pub has_spoiler: bool,
}

#[derive(Debug, Clone, Args)]
pub struct AudioArgs {
    #[command(flatten)]
    pub base: MediaBaseArgs,

    /// 音频的 HTTP URL 或 Telegram file_id
    #[arg(long)]
    pub audio: Option<String>,

    /// 要上传的本地音频路径
    #[arg(long)]
    pub audio_file: Option<PathBuf>,

    /// 音频时长（秒）
    #[arg(long)]
    pub duration: Option<u32>,

    /// 演唱者或表演者
    #[arg(long)]
    pub performer: Option<String>,

    /// 音频标题
    #[arg(long)]
    pub title: Option<String>,

    /// 要上传的本地 JPEG 缩略图
    #[arg(long)]
    pub thumbnail_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct DocumentArgs {
    #[command(flatten)]
    pub base: MediaBaseArgs,

    /// 文档的 HTTP URL 或 Telegram file_id
    #[arg(long)]
    pub document: Option<String>,

    /// 要上传的本地文档路径
    #[arg(long)]
    pub document_file: Option<PathBuf>,

    /// 要上传的本地 JPEG 缩略图
    #[arg(long)]
    pub thumbnail_file: Option<PathBuf>,

    /// 禁用 Telegram 的内容类型检测
    #[arg(long)]
    pub disable_content_type_detection: bool,
}

#[derive(Debug, Clone, Args)]
pub struct VideoArgs {
    #[command(flatten)]
    pub base: MediaBaseArgs,

    /// 视频的 HTTP URL 或 Telegram file_id
    #[arg(long)]
    pub video: Option<String>,

    /// 要上传的本地视频路径
    #[arg(long)]
    pub video_file: Option<PathBuf>,

    /// 视频时长（秒）
    #[arg(long)]
    pub duration: Option<u32>,

    /// 视频宽度（像素）
    #[arg(long)]
    pub width: Option<u32>,

    /// 视频高度（像素）
    #[arg(long)]
    pub height: Option<u32>,

    /// 要上传的本地 JPEG 缩略图
    #[arg(long)]
    pub thumbnail_file: Option<PathBuf>,

    /// 视频封面的 HTTP URL 或 Telegram file_id
    #[arg(long)]
    pub cover: Option<String>,

    /// 要上传的本地视频封面路径
    #[arg(long)]
    pub cover_file: Option<PathBuf>,

    /// 视频起始时间戳（秒）
    #[arg(long)]
    pub start_timestamp: Option<u32>,

    /// 将标题显示在媒体上方
    #[arg(long)]
    pub show_caption_above_media: bool,

    /// 对视频添加剧透遮罩
    #[arg(long)]
    pub has_spoiler: bool,

    /// 声明视频支持流式播放
    #[arg(long)]
    pub supports_streaming: bool,
}

#[derive(Debug, Clone, Args)]
pub struct AnimationArgs {
    #[command(flatten)]
    pub base: MediaBaseArgs,

    /// 动画的 HTTP URL 或 Telegram file_id
    #[arg(long)]
    pub animation: Option<String>,

    /// 要上传的本地动画路径
    #[arg(long)]
    pub animation_file: Option<PathBuf>,

    /// 动画时长（秒）
    #[arg(long)]
    pub duration: Option<u32>,

    /// 动画宽度（像素）
    #[arg(long)]
    pub width: Option<u32>,

    /// 动画高度（像素）
    #[arg(long)]
    pub height: Option<u32>,

    /// 要上传的本地 JPEG 缩略图
    #[arg(long)]
    pub thumbnail_file: Option<PathBuf>,

    /// 将标题显示在媒体上方
    #[arg(long)]
    pub show_caption_above_media: bool,

    /// 对动画添加剧透遮罩
    #[arg(long)]
    pub has_spoiler: bool,
}

#[derive(Debug, Clone, Args)]
pub struct VoiceArgs {
    #[command(flatten)]
    pub base: MediaBaseArgs,

    /// 语音文件的 HTTP URL 或 Telegram file_id
    #[arg(long)]
    pub voice: Option<String>,

    /// 要上传的本地语音文件路径
    #[arg(long)]
    pub voice_file: Option<PathBuf>,

    /// 语音时长（秒）
    #[arg(long)]
    pub duration: Option<u32>,
}
