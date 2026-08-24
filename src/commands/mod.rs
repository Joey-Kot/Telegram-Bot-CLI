use std::path::PathBuf;

use serde_json::Value;

use crate::cli::{
    AnimationArgs, AudioArgs, CaptionArgs, Command, DocumentArgs, ForwardMessageArgs,
    ForwardMessagesArgs, MediaBaseArgs, MessageArgs, PhotoArgs, SendContextArgs, SendOptionsArgs,
    TargetArgs, VideoArgs, VoiceArgs,
};
use crate::error::{AppError, Result};
use crate::input::{InputReader, JsonKind, validate_local_file};
use crate::telegram::{ApiResponse, RequestSpec, TelegramClient};

pub async fn execute(client: &TelegramClient, command: Command) -> Result<ApiResponse> {
    let mut input = InputReader::default();
    let request = match command {
        Command::Check => RequestSpec::new("getMe"),
        Command::Message(args) => build_message(args, &mut input).await?,
        Command::ForwardMessage(args) => build_forward_message(args, &mut input).await?,
        Command::ForwardMessages(args) => build_forward_messages(args)?,
        Command::Photo(args) => build_photo(args, &mut input).await?,
        Command::Audio(args) => build_audio(args, &mut input).await?,
        Command::Document(args) => build_document(args, &mut input).await?,
        Command::Video(args) => build_video(args, &mut input).await?,
        Command::Animation(args) => build_animation(args, &mut input).await?,
        Command::Voice(args) => build_voice(args, &mut input).await?,
    };

    client.execute(request).await
}

async fn build_message(args: MessageArgs, input: &mut InputReader) -> Result<RequestSpec> {
    ensure_exclusive(
        args.parse_mode.is_some(),
        has_json_pair(&args.entities_json, &args.entities_json_file),
        "--parse-mode 不能与 --entities-json 或 --entities-json-file 一起使用",
    )?;

    let text = input
        .required_text(&args.text, &args.text_file, "--text 或 --text-file")
        .await?;

    let mut request = RequestSpec::new("sendMessage");
    add_target(&mut request, &args.target);
    add_send_context(&mut request, &args.context);
    add_send_options(&mut request, &args.send_options, input).await?;
    request.insert_string("text", text);

    if let Some(parse_mode) = args.parse_mode {
        request.insert_string("parse_mode", parse_mode);
    }
    add_optional_json(
        &mut request,
        "entities",
        &args.entities_json,
        &args.entities_json_file,
        "--entities-json 或 --entities-json-file",
        JsonKind::Array,
        input,
    )
    .await?;
    add_optional_json(
        &mut request,
        "link_preview_options",
        &args.link_preview_options_json,
        &args.link_preview_options_json_file,
        "--link-preview-options-json 或 --link-preview-options-json-file",
        JsonKind::Object,
        input,
    )
    .await?;

    Ok(request)
}

async fn build_forward_message(
    args: ForwardMessageArgs,
    input: &mut InputReader,
) -> Result<RequestSpec> {
    let mut request = RequestSpec::new("forwardMessage");
    add_target(&mut request, &args.target);
    request.insert_string("from_chat_id", args.from_chat_id);
    request.insert_i64("message_id", args.message_id);
    add_optional_u32(
        &mut request,
        "video_start_timestamp",
        args.video_start_timestamp,
    );
    request.insert_true("disable_notification", args.disable_notification);
    request.insert_true("protect_content", args.protect_content);
    add_optional_string(&mut request, "message_effect_id", args.message_effect_id);
    add_optional_json(
        &mut request,
        "suggested_post_parameters",
        &args.suggested_post_parameters_json,
        &args.suggested_post_parameters_json_file,
        "--suggested-post-parameters-json 或 --suggested-post-parameters-json-file",
        JsonKind::Object,
        input,
    )
    .await?;

    Ok(request)
}

fn build_forward_messages(args: ForwardMessagesArgs) -> Result<RequestSpec> {
    if args.message_id.is_empty() || args.message_id.len() > 100 {
        return Err(AppError::Input(
            "--message-id 必须提供 1 到 100 次".to_owned(),
        ));
    }
    if args.message_id.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(AppError::Input(
            "--message-id 必须按严格递增顺序提供，且不能重复".to_owned(),
        ));
    }

    let mut request = RequestSpec::new("forwardMessages");
    add_target(&mut request, &args.target);
    request.insert_string("from_chat_id", args.from_chat_id);
    request.insert(
        "message_ids",
        Value::Array(
            args.message_id
                .into_iter()
                .map(|id| Value::Number(id.into()))
                .collect(),
        ),
    );
    request.insert_true("disable_notification", args.disable_notification);
    request.insert_true("protect_content", args.protect_content);

    Ok(request)
}

async fn build_photo(args: PhotoArgs, input: &mut InputReader) -> Result<RequestSpec> {
    let source = required_media_source(&args.photo, &args.photo_file, "--photo 或 --photo-file")?;
    let mut request = build_media_base("sendPhoto", &args.base, input).await?;
    add_media_source(&mut request, "photo", source);
    request.insert_true("show_caption_above_media", args.show_caption_above_media);
    request.insert_true("has_spoiler", args.has_spoiler);

    Ok(request)
}

async fn build_audio(args: AudioArgs, input: &mut InputReader) -> Result<RequestSpec> {
    let source = required_media_source(&args.audio, &args.audio_file, "--audio 或 --audio-file")?;
    let mut request = build_media_base("sendAudio", &args.base, input).await?;
    let source_is_local = source.is_local();
    add_media_source(&mut request, "audio", source);
    add_optional_u32(&mut request, "duration", args.duration);
    add_optional_string(&mut request, "performer", args.performer);
    add_optional_string(&mut request, "title", args.title);
    add_thumbnail(&mut request, args.thumbnail_file, source_is_local)?;

    Ok(request)
}

async fn build_document(args: DocumentArgs, input: &mut InputReader) -> Result<RequestSpec> {
    let source = required_media_source(
        &args.document,
        &args.document_file,
        "--document 或 --document-file",
    )?;
    let mut request = build_media_base("sendDocument", &args.base, input).await?;
    let source_is_local = source.is_local();
    add_media_source(&mut request, "document", source);
    add_thumbnail(&mut request, args.thumbnail_file, source_is_local)?;
    request.insert_true(
        "disable_content_type_detection",
        args.disable_content_type_detection,
    );

    Ok(request)
}

async fn build_video(args: VideoArgs, input: &mut InputReader) -> Result<RequestSpec> {
    let source = required_media_source(&args.video, &args.video_file, "--video 或 --video-file")?;
    let cover = optional_media_source(&args.cover, &args.cover_file, "--cover 或 --cover-file")?;
    let mut request = build_media_base("sendVideo", &args.base, input).await?;
    let source_is_local = source.is_local();
    add_media_source(&mut request, "video", source);
    add_optional_u32(&mut request, "duration", args.duration);
    add_optional_u32(&mut request, "width", args.width);
    add_optional_u32(&mut request, "height", args.height);
    add_thumbnail(&mut request, args.thumbnail_file, source_is_local)?;
    if let Some(cover) = cover {
        add_media_source(&mut request, "cover", cover);
    }
    add_optional_u32(&mut request, "start_timestamp", args.start_timestamp);
    request.insert_true("show_caption_above_media", args.show_caption_above_media);
    request.insert_true("has_spoiler", args.has_spoiler);
    request.insert_true("supports_streaming", args.supports_streaming);

    Ok(request)
}

async fn build_animation(args: AnimationArgs, input: &mut InputReader) -> Result<RequestSpec> {
    let source = required_media_source(
        &args.animation,
        &args.animation_file,
        "--animation 或 --animation-file",
    )?;
    let mut request = build_media_base("sendAnimation", &args.base, input).await?;
    let source_is_local = source.is_local();
    add_media_source(&mut request, "animation", source);
    add_optional_u32(&mut request, "duration", args.duration);
    add_optional_u32(&mut request, "width", args.width);
    add_optional_u32(&mut request, "height", args.height);
    add_thumbnail(&mut request, args.thumbnail_file, source_is_local)?;
    request.insert_true("show_caption_above_media", args.show_caption_above_media);
    request.insert_true("has_spoiler", args.has_spoiler);

    Ok(request)
}

async fn build_voice(args: VoiceArgs, input: &mut InputReader) -> Result<RequestSpec> {
    let source = required_media_source(&args.voice, &args.voice_file, "--voice 或 --voice-file")?;
    let mut request = build_media_base("sendVoice", &args.base, input).await?;
    add_media_source(&mut request, "voice", source);
    add_optional_u32(&mut request, "duration", args.duration);

    Ok(request)
}

async fn build_media_base(
    method: &'static str,
    args: &MediaBaseArgs,
    input: &mut InputReader,
) -> Result<RequestSpec> {
    let mut request = RequestSpec::new(method);
    add_target(&mut request, &args.target);
    add_send_context(&mut request, &args.context);
    add_send_options(&mut request, &args.send_options, input).await?;
    add_caption(&mut request, &args.caption, input).await?;
    Ok(request)
}

fn add_target(request: &mut RequestSpec, args: &TargetArgs) {
    request.insert_string("chat_id", args.chat_id.clone());
    add_optional_i64(request, "message_thread_id", args.message_thread_id);
    add_optional_i64(
        request,
        "direct_messages_topic_id",
        args.direct_messages_topic_id,
    );
}

fn add_send_context(request: &mut RequestSpec, args: &SendContextArgs) {
    add_optional_string(
        request,
        "business_connection_id",
        args.business_connection_id.clone(),
    );
    add_optional_i64(request, "receiver_user_id", args.receiver_user_id);
    add_optional_string(request, "callback_query_id", args.callback_query_id.clone());
}

async fn add_send_options(
    request: &mut RequestSpec,
    args: &SendOptionsArgs,
    input: &mut InputReader,
) -> Result<()> {
    request.insert_true("disable_notification", args.disable_notification);
    request.insert_true("protect_content", args.protect_content);
    request.insert_true("allow_paid_broadcast", args.allow_paid_broadcast);
    add_optional_string(request, "message_effect_id", args.message_effect_id.clone());
    add_optional_json(
        request,
        "suggested_post_parameters",
        &args.suggested_post_parameters_json,
        &args.suggested_post_parameters_json_file,
        "--suggested-post-parameters-json 或 --suggested-post-parameters-json-file",
        JsonKind::Object,
        input,
    )
    .await?;
    add_optional_json(
        request,
        "reply_parameters",
        &args.reply_parameters_json,
        &args.reply_parameters_json_file,
        "--reply-parameters-json 或 --reply-parameters-json-file",
        JsonKind::Object,
        input,
    )
    .await?;
    add_optional_json(
        request,
        "reply_markup",
        &args.reply_markup_json,
        &args.reply_markup_json_file,
        "--reply-markup-json 或 --reply-markup-json-file",
        JsonKind::Object,
        input,
    )
    .await?;

    Ok(())
}

async fn add_caption(
    request: &mut RequestSpec,
    args: &CaptionArgs,
    input: &mut InputReader,
) -> Result<()> {
    ensure_exclusive(
        args.parse_mode.is_some(),
        has_json_pair(
            &args.caption_entities_json,
            &args.caption_entities_json_file,
        ),
        "--parse-mode 不能与 --caption-entities-json 或 --caption-entities-json-file 一起使用",
    )?;
    let caption = input
        .optional_text(
            &args.caption,
            &args.caption_file,
            "--caption 或 --caption-file",
        )
        .await?;
    if caption.is_none()
        && (args.parse_mode.is_some()
            || has_json_pair(
                &args.caption_entities_json,
                &args.caption_entities_json_file,
            ))
    {
        return Err(AppError::Input(
            "--parse-mode 和 --caption-entities-* 需要同时提供 --caption 或 --caption-file"
                .to_owned(),
        ));
    }

    if let Some(caption) = caption {
        request.insert_string("caption", caption);
    }
    if let Some(parse_mode) = &args.parse_mode {
        request.insert_string("parse_mode", parse_mode.clone());
    }
    add_optional_json(
        request,
        "caption_entities",
        &args.caption_entities_json,
        &args.caption_entities_json_file,
        "--caption-entities-json 或 --caption-entities-json-file",
        JsonKind::Array,
        input,
    )
    .await?;

    Ok(())
}

async fn add_optional_json(
    request: &mut RequestSpec,
    field_name: &'static str,
    inline: &Option<String>,
    file: &Option<PathBuf>,
    label: &'static str,
    expected_kind: JsonKind,
    input: &mut InputReader,
) -> Result<()> {
    if let Some(value) = input
        .optional_json(inline, file, label, expected_kind)
        .await?
    {
        request.insert(field_name, value);
    }
    Ok(())
}

fn ensure_exclusive(first: bool, second: bool, message: &'static str) -> Result<()> {
    if first && second {
        Err(AppError::Input(message.to_owned()))
    } else {
        Ok(())
    }
}

fn has_json_pair(inline: &Option<String>, file: &Option<PathBuf>) -> bool {
    inline.is_some() || file.is_some()
}

fn add_optional_string(request: &mut RequestSpec, field_name: &'static str, value: Option<String>) {
    if let Some(value) = value {
        request.insert_string(field_name, value);
    }
}

fn add_optional_i64(request: &mut RequestSpec, field_name: &'static str, value: Option<i64>) {
    if let Some(value) = value {
        request.insert_i64(field_name, value);
    }
}

fn add_optional_u32(request: &mut RequestSpec, field_name: &'static str, value: Option<u32>) {
    if let Some(value) = value {
        request.insert_u32(field_name, value);
    }
}

#[derive(Debug)]
enum MediaSource {
    Remote(String),
    Local(PathBuf),
}

impl MediaSource {
    const fn is_local(&self) -> bool {
        matches!(self, Self::Local(_))
    }
}

fn required_media_source(
    remote: &Option<String>,
    local: &Option<PathBuf>,
    label: &'static str,
) -> Result<MediaSource> {
    optional_media_source(remote, local, label)?
        .ok_or_else(|| AppError::Input(format!("必须提供 {label}")))
}

fn optional_media_source(
    remote: &Option<String>,
    local: &Option<PathBuf>,
    label: &'static str,
) -> Result<Option<MediaSource>> {
    match (remote, local) {
        (Some(_), Some(_)) => Err(AppError::Input(format!("{label} 只能提供其中一个"))),
        (Some(value), None) => Ok(Some(MediaSource::Remote(value.clone()))),
        (None, Some(path)) => {
            validate_local_file(path)?;
            Ok(Some(MediaSource::Local(path.clone())))
        }
        (None, None) => Ok(None),
    }
}

fn add_media_source(request: &mut RequestSpec, field_name: &'static str, source: MediaSource) {
    match source {
        MediaSource::Remote(value) => request.insert_string(field_name, value),
        MediaSource::Local(path) => request.add_file(field_name, path),
    }
}

fn add_thumbnail(
    request: &mut RequestSpec,
    thumbnail: Option<PathBuf>,
    main_media_is_local: bool,
) -> Result<()> {
    let Some(path) = thumbnail else {
        return Ok(());
    };
    if !main_media_is_local {
        return Err(AppError::Input(
            "--thumbnail-file 只能与对应的 --*-file 本地主媒体一起使用".to_owned(),
        ));
    }
    validate_local_file(&path)?;
    request.add_file("thumbnail", path);
    Ok(())
}
