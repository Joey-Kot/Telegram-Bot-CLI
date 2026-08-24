English | [简体中文](README_ZH.md)

# tgpush

`tgpush` is an automation-oriented command-line client for the Telegram Bot API. It covers common message sending, forwarding, and media methods, and writes Telegram's JSON responses to stdout unchanged so that shells, CI systems, scripts, and other programs can continue parsing message IDs, error codes, and rate-limit information.

Supported features:

- `getMe` connectivity and token checks
- `sendMessage`
- `forwardMessage` and `forwardMessages`
- `sendPhoto`, `sendAudio`, `sendDocument`, `sendVideo`, `sendAnimation`, and `sendVoice`
- Telegram `file_id` values, HTTP URLs, and local multipart file uploads
- File-based input for text, captions, and complex JSON parameters

## Downloads

| Platform | Download | SHA-256 |
|---|---|---|
| Linux x86_64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-linux-x86_64.tar.gz) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-linux-x86_64.tar.gz.sha256) |
| Linux arm64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-linux-arm64.tar.gz) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-linux-arm64.tar.gz.sha256) |
| Windows x86_64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-windows-x86_64.zip) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-windows-x86_64.zip.sha256) |
| Windows arm64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-windows-arm64.zip) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-windows-arm64.zip.sha256) |
| macOS x86_64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-macos-x86_64.tar.gz) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-macos-x86_64.tar.gz.sha256) |
| macOS arm64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-macos-arm64.tar.gz) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-macos-arm64.tar.gz.sha256) |

## Configuring the Bot Token

`tgpush` reads the token exclusively from environment variables and does not provide a `--token` option. This keeps the token out of command-line arguments, shell history, and process argument lists.

| Environment variable | Required | Description |
|---|---:|---|
| `TELEGRAM_BOT_TOKEN` | Yes | The Bot Token obtained from BotFather. |
| `TELEGRAM_BOT_API_BASE_URL` | No | Bot API base URL. Defaults to `https://api.telegram.org`. It can be used with a self-hosted Bot API Server or a test server. |

Linux and macOS:

```bash
export TELEGRAM_BOT_TOKEN='123456789:AAExampleBotToken_ReplaceMe1234567890'
export TELEGRAM_BOT_API_BASE_URL='https://api.telegram.org'
```

PowerShell:

```powershell
$env:TELEGRAM_BOT_TOKEN = '123456789:AAExampleBotToken_ReplaceMe1234567890'
$env:TELEGRAM_BOT_API_BASE_URL = 'https://api.telegram.org'
```

Windows Command Prompt:

```batch
set "TELEGRAM_BOT_TOKEN=123456789:AAExampleBotToken_ReplaceMe1234567890"
set "TELEGRAM_BOT_API_BASE_URL=https://api.telegram.org"
```

If you configure a self-hosted server, use `http://` only on a controlled local network. The Telegram token appears in the Bot API request path, so never send it to an untrusted HTTP service.

## Quick Start

First, check connectivity and validate the token:

```bash
tgpush check
```

Send a plain-text message:

```bash
tgpush message \
  --chat-id '@example_channel' \
  --text 'Hello from tgpush'
```

Send a local photo:

```bash
tgpush photo \
  --chat-id -1001234567890 \
  --photo-file ./notice.jpg \
  --caption 'Photo caption'
```

Forward multiple messages:

```bash
tgpush forward-messages \
  --chat-id -1001234567890 \
  --from-chat-id -1009876543210 \
  --message-id 10 \
  --message-id 11 \
  --message-id 15
```

## Command Overview

| Subcommand | Telegram method |
|---|---|
| `check` | `getMe` |
| `message` | `sendMessage` |
| `forward-message` | `forwardMessage` |
| `forward-messages` | `forwardMessages` |
| `photo` | `sendPhoto` |
| `audio` | `sendAudio` |
| `document` | `sendDocument` |
| `video` | `sendVideo` |
| `animation` | `sendAnimation` |
| `voice` | `sendVoice` |

All operational arguments use long options only, such as `--chat-id` and `--photo-file`. `--help` and `--version` are also available only in their long forms.

## Target Chat and Sending Options

`message` and all media commands share the following target options:

| Option | API field | Description |
|---|---|---|
| `--chat-id <ID>` | `chat_id` | Required. A chat ID, a negative group/channel ID, or an `@username`. |
| `--message-thread-id <ID>` | `message_thread_id` | Forum topic ID. |
| `--direct-messages-topic-id <ID>` | `direct_messages_topic_id` | Direct Messages topic ID. |
| `--business-connection-id <ID>` | `business_connection_id` | Business connection ID. |
| `--receiver-user-id <ID>` | `receiver_user_id` | Recipient user ID for an ephemeral message. |
| `--callback-query-id <ID>` | `callback_query_id` | ID of the Callback Query that triggered an ephemeral message. |

`message` and all media commands also share these sending options:

| Option | API field | Description |
|---|---|---|
| `--disable-notification` | `disable_notification` | Send silently. |
| `--protect-content` | `protect_content` | Protect the content from forwarding and saving. |
| `--allow-paid-broadcast` | `allow_paid_broadcast` | Allow paid broadcasts, which may consume Telegram Stars. |
| `--message-effect-id <ID>` | `message_effect_id` | Message effect ID for private chats. |
| `--suggested-post-parameters-json <JSON>` | `suggested_post_parameters` | JSON object. |
| `--suggested-post-parameters-json-file <PATH>` | `suggested_post_parameters` | JSON file. |
| `--reply-parameters-json <JSON>` | `reply_parameters` | JSON object. |
| `--reply-parameters-json-file <PATH>` | `reply_parameters` | JSON file. |
| `--reply-markup-json <JSON>` | `reply_markup` | JSON object, such as an Inline Keyboard. |
| `--reply-markup-json-file <PATH>` | `reply_markup` | JSON file. |

Each `--*-json` option is mutually exclusive with its matching `--*-json-file` option.

## `check`

```bash
tgpush check
```

Calls `getMe`. It checks the network connection, TLS, the Bot API address, and token validity without sending a message.

## `message`

Required options:

| Option | API field |
|---|---|
| `--chat-id <ID>` | `chat_id` |
| `--text <TEXT>` or `--text-file <PATH>` | `text` |

Command-specific optional arguments:

| Option | API field |
|---|---|
| `--parse-mode <MODE>` | `parse_mode` |
| `--entities-json <JSON>` | `entities` |
| `--entities-json-file <PATH>` | `entities` |
| `--link-preview-options-json <JSON>` | `link_preview_options` |
| `--link-preview-options-json-file <PATH>` | `link_preview_options` |

`--parse-mode` cannot be used together with `--entities-json` or `--entities-json-file`.

Example:

```bash
tgpush message \
  --chat-id '@example_channel' \
  --text 'This is *MarkdownV2* text' \
  --parse-mode MarkdownV2
```

## `forward-message`

| Option | Required | API field |
|---|---:|---|
| `--chat-id <ID>` | Yes | `chat_id` |
| `--from-chat-id <ID>` | Yes | `from_chat_id` |
| `--message-id <ID>` | Yes | `message_id` |
| `--message-thread-id <ID>` | No | `message_thread_id` |
| `--direct-messages-topic-id <ID>` | No | `direct_messages_topic_id` |
| `--video-start-timestamp <SECONDS>` | No | `video_start_timestamp` |
| `--disable-notification` | No | `disable_notification` |
| `--protect-content` | No | `protect_content` |
| `--message-effect-id <ID>` | No | `message_effect_id` |
| `--suggested-post-parameters-json <JSON>` | No | `suggested_post_parameters` |
| `--suggested-post-parameters-json-file <PATH>` | No | `suggested_post_parameters` |

Service messages and protected content cannot be forwarded. Telegram reports the result in the JSON response.

## `forward-messages`

| Option | Required | API field |
|---|---:|---|
| `--chat-id <ID>` | Yes | `chat_id` |
| `--from-chat-id <ID>` | Yes | `from_chat_id` |
| `--message-id <ID>` | Yes, repeatable | Collected into `message_ids` |
| `--message-thread-id <ID>` | No | `message_thread_id` |
| `--direct-messages-topic-id <ID>` | No | `direct_messages_topic_id` |
| `--disable-notification` | No | `disable_notification` |
| `--protect-content` | No | `protect_content` |

`--message-id` must be provided between 1 and 100 times in strictly increasing order. The CLI does not sort the values automatically. Telegram may skip messages that cannot be found or forwarded, so the result array in a successful response may be shorter than the input list.

## Media Sources

Each media command requires exactly one remote value or local file:

| Command | URL or `file_id` | Local upload |
|---|---|---|
| `photo` | `--photo <VALUE>` | `--photo-file <PATH>` |
| `audio` | `--audio <VALUE>` | `--audio-file <PATH>` |
| `document` | `--document <VALUE>` | `--document-file <PATH>` |
| `video` | `--video <VALUE>` | `--video-file <PATH>` |
| `animation` | `--animation <VALUE>` | `--animation-file <PATH>` |
| `voice` | `--voice <VALUE>` | `--voice-file <PATH>` |

Examples:

```bash
# Use a Telegram file_id already owned by the current Bot
tgpush photo --chat-id '@example_channel' --photo 'AgACAgQAAxkBAAIB...'

# Let Telegram download the file from a URL
tgpush video --chat-id '@example_channel' --video 'https://example.com/demo.mp4'

# Upload a local file
tgpush document --chat-id '@example_channel' --document-file ./report.pdf
```

Values passed to `--photo`, `--video`, and similar options are forwarded to Telegram unchanged, so they must represent either a URL or a `file_id`. Local paths must use the corresponding `--*-file` option; the CLI does not guess whether a string is a file path. A `file_id` is scoped to an individual Bot and cannot be reused across Bots.

Local media is uploaded as a streaming `multipart/form-data` request without loading the entire file into memory.

## Media Captions

All media commands share these options:

| Option | API field |
|---|---|
| `--caption <TEXT>` | `caption` |
| `--caption-file <PATH>` | `caption` |
| `--parse-mode <MODE>` | `parse_mode` |
| `--caption-entities-json <JSON>` | `caption_entities` |
| `--caption-entities-json-file <PATH>` | `caption_entities` |

`--caption` and `--caption-file` are mutually exclusive. `--parse-mode` and `--caption-entities-*` are also mutually exclusive, and either one requires a caption to be provided.

## Media-Specific Options

### `photo`

| Option | API field |
|---|---|
| `--photo <VALUE>` / `--photo-file <PATH>` | `photo` |
| `--show-caption-above-media` | `show_caption_above_media` |
| `--has-spoiler` | `has_spoiler` |

### `audio`

| Option | API field |
|---|---|
| `--audio <VALUE>` / `--audio-file <PATH>` | `audio` |
| `--duration <SECONDS>` | `duration` |
| `--performer <TEXT>` | `performer` |
| `--title <TEXT>` | `title` |
| `--thumbnail-file <PATH>` | `thumbnail` |

### `document`

| Option | API field |
|---|---|
| `--document <VALUE>` / `--document-file <PATH>` | `document` |
| `--thumbnail-file <PATH>` | `thumbnail` |
| `--disable-content-type-detection` | `disable_content_type_detection` |

### `video`

| Option | API field |
|---|---|
| `--video <VALUE>` / `--video-file <PATH>` | `video` |
| `--duration <SECONDS>` | `duration` |
| `--width <PIXELS>` | `width` |
| `--height <PIXELS>` | `height` |
| `--thumbnail-file <PATH>` | `thumbnail` |
| `--cover <VALUE>` / `--cover-file <PATH>` | `cover` |
| `--start-timestamp <SECONDS>` | `start_timestamp` |
| `--show-caption-above-media` | `show_caption_above_media` |
| `--has-spoiler` | `has_spoiler` |
| `--supports-streaming` | `supports_streaming` |

### `animation`

| Option | API field |
|---|---|
| `--animation <VALUE>` / `--animation-file <PATH>` | `animation` |
| `--duration <SECONDS>` | `duration` |
| `--width <PIXELS>` | `width` |
| `--height <PIXELS>` | `height` |
| `--thumbnail-file <PATH>` | `thumbnail` |
| `--show-caption-above-media` | `show_caption_above_media` |
| `--has-spoiler` | `has_spoiler` |

### `voice`

| Option | API field |
|---|---|
| `--voice <VALUE>` / `--voice-file <PATH>` | `voice` |
| `--duration <SECONDS>` | `duration` |

Thumbnails must be uploaded as new local files and cannot reuse a `file_id`. `--thumbnail-file` is accepted only when the main media is also uploaded through the corresponding `--*-file` option. Telegram requires thumbnails to be JPEG files smaller than 200 KB, with both dimensions no greater than 320 pixels. The CLI validates only the path; Telegram performs the final format validation.

## Text, JSON, and Shell Escaping

The CLI does not interpret backslash escapes and does not automatically escape Markdown, MarkdownV2, or HTML.

The following Bash command sends the literal characters `\n` to Telegram instead of a newline:

```bash
tgpush message \
  --chat-id '@example_channel' \
  --text 'line one\nline two'
```

To send real newlines, use a multiline shell string or a text file:

```bash
tgpush message \
  --chat-id '@example_channel' \
  --text-file ./message.txt
```

You can also read from stdin:

```bash
printf 'first line\nsecond line\n' | tgpush message \
  --chat-id '@example_channel' \
  --text-file -
```

PowerShell example:

```powershell
tgpush message `
  --chat-id '@example_channel' `
  --text-file .\message.txt
```

For complex parameters, prefer JSON files to avoid shell-escaping issues:

```json
{
  "inline_keyboard": [
    [
      {
        "text": "Open website",
        "url": "https://example.com"
      }
    ]
  ]
}
```

```bash
tgpush message \
  --chat-id '@example_channel' \
  --text 'Choose an option:' \
  --reply-markup-json-file ./keyboard.json
```

`--text-file`, `--caption-file`, and all `--*-json-file` options accept `-` to read from stdin. Only one such option may use `-` in a single invocation.

## Output, Errors, and Exit Codes

Telegram Bot API responses are JSON objects. Whenever `tgpush` receives valid Telegram JSON, it writes the HTTP response body to stdout byte for byte: it does not reserialize, pretty-print, prefix, or append an extra newline.

Successful response example:

```json
{"ok":true,"result":{"message_id":42,"chat":{"id":-1001234567890}}}
```

Telegram API errors are also written to stdout:

```json
{"ok":false,"error_code":400,"description":"Bad Request: chat not found"}
```

Callers should therefore always read stdout and inspect `ok`, `result`, `description`, `error_code`, and `parameters`. For example:

```bash
tgpush message --chat-id '@example_channel' --text 'hello' | jq '.result.message_id'
```

Exit codes:

| Scenario | stdout | stderr | Exit code |
|---|---|---|---:|
| Telegram returns `ok: true` | Original JSON | Empty | `0` |
| Telegram returns `ok: false` | Original JSON | Empty | `1` |
| HTTP error with a Telegram JSON body | Original JSON | Empty | `1` |
| Network, TLS, timeout, or non-JSON gateway response | Empty | Local diagnostic | `1` |
| Invalid arguments, environment variables, local files, or input JSON | Empty | Local diagnostic | `2` |
| `--help` or `--version` | Help or version text | Empty | `0` |

Local diagnostics never print the token or a request URL containing the token. Telegram's default API does not echo the token in JSON responses. If an untrusted self-hosted API address is configured, any JSON returned by that server is still written unchanged.

## Telegram Limits

Bot API limits are controlled by Telegram and may change over time. Important limits listed in the provided API documentation include:

- Photos sent by URL through the cloud Bot API are generally limited to 5 MB, while other content sent by URL is generally limited to 20 MB.
- Local photo uploads are generally limited to 10 MB, while other local media uploads are generally limited to 50 MB.
- `sendDocument` currently guarantees URL-based delivery only for PDF and ZIP files.
- Sending voice messages by URL is more restrictive than uploading local files; local OGG/OPUS, MP3, or M4A files are more reliable.
- A self-hosted Telegram Bot API Server may allow larger local uploads.

`tgpush` does not hard-code these size or media-format limits on the client side, so it does not block self-hosted Bot API Servers or future API updates. The server returns the authoritative JSON result.

## Building from Source

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked --release
```

The executable is written to:

```text
target/release/tgpush
```

On Windows, the path is `target/release/tgpush.exe`.

## License

This project is licensed under the [MIT License](LICENSE). See [NOTICE](NOTICE) for full copyright information.
