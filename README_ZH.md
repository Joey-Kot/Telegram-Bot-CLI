[English](README.md) | 简体中文

# tgpush

`tgpush` 是一个面向自动化场景的 Telegram Bot API 命令行客户端。覆盖常用的发消息、转发和媒体发送方法，并把 Telegram 返回的 JSON 响应原样输出，便于 shell、CI、脚本和其他程序继续解析消息 ID、错误码或限流信息。

支持：

- `getMe` 连通性和 Token 检查
- `sendMessage`
- `deleteMessage`
- `forwardMessage`、`forwardMessages`
- `sendPhoto`、`sendAudio`、`sendDocument`、`sendVideo`、`sendAnimation`、`sendVoice`
- Telegram `file_id`、HTTP URL 和本地 multipart 文件上传
- 文本、标题和复杂 JSON 参数的文件输入

## 下载

| 平台 | 下载 | SHA-256 |
|---|---|---|
| Linux x86_64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-linux-x86_64.tar.gz) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-linux-x86_64.tar.gz.sha256) |
| Linux arm64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-linux-arm64.tar.gz) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-linux-arm64.tar.gz.sha256) |
| Windows x86_64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-windows-x86_64.zip) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-windows-x86_64.zip.sha256) |
| Windows arm64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-windows-arm64.zip) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-windows-arm64.zip.sha256) |
| macOS x86_64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-macos-x86_64.tar.gz) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-macos-x86_64.tar.gz.sha256) |
| macOS arm64 | [download](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-macos-arm64.tar.gz) | [sha256](https://github.com/Joey-Kot/Telegram-Bot-CLI/releases/download/Latest/tgpush-macos-arm64.tar.gz.sha256) |

## 配置 Bot Token

`tgpush` 只通过环境变量读取 Token，不提供 `--token` 参数，因此 Token 不会因命令行参数、shell 历史记录或进程参数列表而暴露。

| 环境变量 | 是否必填 | 说明 |
|---|---:|---|
| `TELEGRAM_BOT_TOKEN` | 是 | 通过 BotFather 获取的 Bot Token。 |
| `TELEGRAM_BOT_API_BASE_URL` | 否 | Bot API 基础地址，默认 `https://api.telegram.org`。可用于自建 Bot API Server 或测试服务器。 |

Linux 和 macOS：

```bash
export TELEGRAM_BOT_TOKEN='123456789:AAExampleBotToken_ReplaceMe1234567890'
export TELEGRAM_BOT_API_BASE_URL='https://api.telegram.org'
```

PowerShell：

```powershell
$env:TELEGRAM_BOT_TOKEN = '123456789:AAExampleBotToken_ReplaceMe1234567890'
$env:TELEGRAM_BOT_API_BASE_URL = 'https://api.telegram.org'
```

Windows 命令提示符：

```batch
set "TELEGRAM_BOT_TOKEN=123456789:AAExampleBotToken_ReplaceMe1234567890"
set "TELEGRAM_BOT_API_BASE_URL=https://api.telegram.org"
```

如果设置自建服务器地址，`http://` 仅建议用于受控的本地网络。Telegram Token 会出现在 Bot API 请求路径中，不要把 Token 发送到不可信的 HTTP 服务。

## 代理

使用全局参数 `--proxy <URL>` 可在子命令前或后为当前 Bot API 请求指定一个代理：

```bash
tgpush --proxy 'http://127.0.0.1:8080' check
tgpush check --proxy 'socks5://proxy-user:proxy-password@127.0.0.1:1080'
```

支持以下 URL 协议：

| 协议 | 说明 |
|---|---|
| `http://` | URL 中可通过用户名和密码使用 HTTP 代理 Basic 认证。 |
| `https://` | 连接 HTTP 代理时使用 TLS；URL 中的用户名和密码使用 Basic 认证。 |
| `socks4://` / `socks4a://` | 分别在本机 / 代理端解析 DNS 的 SOCKS4。URL 用户名会作为 SOCKS4 用户 ID 发送；SOCKS4 不支持密码认证。 |
| `socks5://` / `socks5h://` | 分别在本机 / 代理端解析 DNS 的 SOCKS5，支持用户名密码认证。 |
| `socks://` | `socks5://` 的别名。 |

认证信息写在 URL 中，例如 `https://user:password@proxy.example:8443` 或 `socks5://user:password@proxy.example:1080`。用户名或密码包含保留字符时需使用百分号编码。SOCKS4/4a 仅接受用户 ID，例如 `socks4://user@proxy.example:1080`；由于 SOCKS4 不支持密码认证，带密码的 URL 会被拒绝。需要用户名密码认证时请使用 SOCKS5。

代理 URL（包括其中的认证信息）会作为命令行参数出现，可能进入 shell 历史记录或被进程列表看到。请根据运行环境妥善使用。

## 快速开始

先检查连通性和 Token：

```bash
tgpush check
```

发送纯文本消息：

```bash
tgpush message \
  --chat-id '@example_channel' \
  --text 'Hello from tgpush'
```

发送本地图片：

```bash
tgpush photo \
  --chat-id -1001234567890 \
  --photo-file ./notice.jpg \
  --caption '图片标题'
```

转发多条消息：

```bash
tgpush forward-messages \
  --chat-id -1001234567890 \
  --from-chat-id -1009876543210 \
  --message-id 10 \
  --message-id 11 \
  --message-id 15
```

## 命令总览

| 子命令 | 调用的 Telegram 方法 |
|---|---|
| `check` | `getMe` |
| `message` | `sendMessage` |
| `delete` | `deleteMessage` |
| `forward-message` | `forwardMessage` |
| `forward-messages` | `forwardMessages` |
| `photo` | `sendPhoto` |
| `audio` | `sendAudio` |
| `document` | `sendDocument` |
| `video` | `sendVideo` |
| `animation` | `sendAnimation` |
| `voice` | `sendVoice` |

所有业务参数都只使用长参数，例如 `--chat-id`、`--photo-file`。`--help` 和 `--version` 也只提供长形式。

## 目标聊天和发送参数

`message` 及所有媒体命令共享以下目标参数：

| 参数 | API 字段 | 说明 |
|---|---|---|
| `--chat-id <ID>` | `chat_id` | 必填。聊天 ID、负数群组/频道 ID 或 `@username`。 |
| `--message-thread-id <ID>` | `message_thread_id` | Forum 话题 ID。 |
| `--direct-messages-topic-id <ID>` | `direct_messages_topic_id` | Direct Messages 话题 ID。 |
| `--business-connection-id <ID>` | `business_connection_id` | Business 连接 ID。 |
| `--receiver-user-id <ID>` | `receiver_user_id` | 临时消息的接收用户 ID。 |
| `--callback-query-id <ID>` | `callback_query_id` | 触发临时消息的 Callback Query ID。 |

`message` 及所有媒体命令还共享这些发送行为参数：

| 参数 | API 字段 | 说明 |
|---|---|---|
| `--disable-notification` | `disable_notification` | 静默发送。 |
| `--protect-content` | `protect_content` | 保护内容，禁止转发和保存。 |
| `--allow-paid-broadcast` | `allow_paid_broadcast` | 允许付费广播，可能消耗 Telegram Stars。 |
| `--message-effect-id <ID>` | `message_effect_id` | 私聊消息特效 ID。 |
| `--suggested-post-parameters-json <JSON>` | `suggested_post_parameters` | JSON 对象。 |
| `--suggested-post-parameters-json-file <PATH>` | `suggested_post_parameters` | JSON 文件。 |
| `--reply-parameters-json <JSON>` | `reply_parameters` | JSON 对象。 |
| `--reply-parameters-json-file <PATH>` | `reply_parameters` | JSON 文件。 |
| `--reply-markup-json <JSON>` | `reply_markup` | JSON 对象，例如 Inline Keyboard。 |
| `--reply-markup-json-file <PATH>` | `reply_markup` | JSON 文件。 |

每组 `--*-json` 和 `--*-json-file` 互斥。

## `check`

```bash
tgpush check
```

调用 `getMe`。它会同时检查网络、TLS、Bot API 地址和 Token 是否有效，不会发送消息。

## `message`

必填参数：

| 参数 | API 字段 |
|---|---|
| `--chat-id <ID>` | `chat_id` |
| `--text <TEXT>` 或 `--text-file <PATH>` | `text` |

专有可选参数：

| 参数 | API 字段 |
|---|---|
| `--parse-mode <MODE>` | `parse_mode` |
| `--entities-json <JSON>` | `entities` |
| `--entities-json-file <PATH>` | `entities` |
| `--link-preview-options-json <JSON>` | `link_preview_options` |
| `--link-preview-options-json-file <PATH>` | `link_preview_options` |

`--parse-mode` 不能与 `--entities-json` 或 `--entities-json-file` 同时使用。

示例：

```bash
tgpush message \
  --chat-id '@example_channel' \
  --text '这是 *MarkdownV2* 文本' \
  --parse-mode MarkdownV2
```

## `delete`

删除指定聊天中的单条消息：

```bash
tgpush delete \
  --chat-id '-1001234567890' \
  --message-id 123
```

| 参数 | 必填 | API 字段 |
|---|---:|---|
| `--chat-id <ID>` | 是 | `chat_id`，支持聊天数字 ID 或 `@username` |
| `--message-id <ID>` | 是 | `message_id`，必须为正整数 |

成功时原样输出 Telegram 的 `{"ok":true,"result":true}` 响应，退出码为 0；API 返回删除失败时，原样输出错误 JSON，退出码为 1。全局 `--proxy` 参数同样适用。

消息删除受 Telegram 的时间和权限限制（通常要求消息发送不满 48 小时）；最终结果由 Telegram 判断。详细规则见 [deleteMessage 官方文档](https://core.telegram.org/bots/api#deletemessage)。

## `forward-message`

| 参数 | 必填 | API 字段 |
|---|---:|---|
| `--chat-id <ID>` | 是 | `chat_id` |
| `--from-chat-id <ID>` | 是 | `from_chat_id` |
| `--message-id <ID>` | 是 | `message_id` |
| `--message-thread-id <ID>` | 否 | `message_thread_id` |
| `--direct-messages-topic-id <ID>` | 否 | `direct_messages_topic_id` |
| `--video-start-timestamp <SECONDS>` | 否 | `video_start_timestamp` |
| `--disable-notification` | 否 | `disable_notification` |
| `--protect-content` | 否 | `protect_content` |
| `--message-effect-id <ID>` | 否 | `message_effect_id` |
| `--suggested-post-parameters-json <JSON>` | 否 | `suggested_post_parameters` |
| `--suggested-post-parameters-json-file <PATH>` | 否 | `suggested_post_parameters` |

服务消息和受保护内容不能被转发；Telegram 会在响应 JSON 中说明结果。

## `forward-messages`

| 参数 | 必填 | API 字段 |
|---|---:|---|
| `--chat-id <ID>` | 是 | `chat_id` |
| `--from-chat-id <ID>` | 是 | `from_chat_id` |
| `--message-id <ID>` | 是，可重复 | 汇总为 `message_ids` |
| `--message-thread-id <ID>` | 否 | `message_thread_id` |
| `--direct-messages-topic-id <ID>` | 否 | `direct_messages_topic_id` |
| `--disable-notification` | 否 | `disable_notification` |
| `--protect-content` | 否 | `protect_content` |

`--message-id` 必须提供 1 到 100 次，并且严格递增。CLI 不会自动排序。Telegram 可能跳过找不到或不能转发的消息，因此成功响应中的结果数组可能短于输入数量。

## 媒体来源

每个媒体命令都要求在远程值和本地文件之间二选一：

| 命令 | URL 或 `file_id` | 本地上传 |
|---|---|---|
| `photo` | `--photo <VALUE>` | `--photo-file <PATH>` |
| `audio` | `--audio <VALUE>` | `--audio-file <PATH>` |
| `document` | `--document <VALUE>` | `--document-file <PATH>` |
| `video` | `--video <VALUE>` | `--video-file <PATH>` |
| `animation` | `--animation <VALUE>` | `--animation-file <PATH>` |
| `voice` | `--voice <VALUE>` | `--voice-file <PATH>` |

例如：

```bash
# 使用当前 Bot 已拥有的 Telegram file_id
tgpush photo --chat-id '@example_channel' --photo 'AgACAgQAAxkBAAIB...'

# 让 Telegram 从 URL 下载
tgpush video --chat-id '@example_channel' --video 'https://example.com/demo.mp4'

# 上传本地文件
tgpush document --chat-id '@example_channel' --document-file ./report.pdf
```

`--photo`、`--video` 等值会原样交给 Telegram，因此它们只能表示 URL 或 `file_id`。本地路径必须使用相应的 `--*-file` 参数；CLI 不会猜测某个字符串是否为文件路径。`file_id` 对各个 Bot 独立，不能跨 Bot 使用。

本地媒体采用流式 `multipart/form-data` 上传，不会把整个文件读入内存。

## 媒体标题

所有媒体命令共享：

| 参数 | API 字段 |
|---|---|
| `--caption <TEXT>` | `caption` |
| `--caption-file <PATH>` | `caption` |
| `--parse-mode <MODE>` | `parse_mode` |
| `--caption-entities-json <JSON>` | `caption_entities` |
| `--caption-entities-json-file <PATH>` | `caption_entities` |

`--caption` 与 `--caption-file` 互斥。`--parse-mode` 与 `--caption-entities-*` 互斥；使用其中任一参数时必须同时提供标题。

## 媒体命令专有参数

### `photo`

| 参数 | API 字段 |
|---|---|
| `--photo <VALUE>` / `--photo-file <PATH>` | `photo` |
| `--show-caption-above-media` | `show_caption_above_media` |
| `--has-spoiler` | `has_spoiler` |

### `audio`

| 参数 | API 字段 |
|---|---|
| `--audio <VALUE>` / `--audio-file <PATH>` | `audio` |
| `--duration <SECONDS>` | `duration` |
| `--performer <TEXT>` | `performer` |
| `--title <TEXT>` | `title` |
| `--thumbnail-file <PATH>` | `thumbnail` |

### `document`

| 参数 | API 字段 |
|---|---|
| `--document <VALUE>` / `--document-file <PATH>` | `document` |
| `--thumbnail-file <PATH>` | `thumbnail` |
| `--disable-content-type-detection` | `disable_content_type_detection` |

### `video`

| 参数 | API 字段 |
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

| 参数 | API 字段 |
|---|---|
| `--animation <VALUE>` / `--animation-file <PATH>` | `animation` |
| `--duration <SECONDS>` | `duration` |
| `--width <PIXELS>` | `width` |
| `--height <PIXELS>` | `height` |
| `--thumbnail-file <PATH>` | `thumbnail` |
| `--show-caption-above-media` | `show_caption_above_media` |
| `--has-spoiler` | `has_spoiler` |

### `voice`

| 参数 | API 字段 |
|---|---|
| `--voice <VALUE>` / `--voice-file <PATH>` | `voice` |
| `--duration <SECONDS>` | `duration` |

缩略图必须使用新的本地文件上传，不能复用 `file_id`。`--thumbnail-file` 只允许在主媒体也通过对应 `--*-file` 上传时使用；Telegram 文档要求 JPEG、小于 200 KB、宽高不超过 320，CLI 只校验路径，最终格式校验由 Telegram 执行。

## 文本、JSON 和 shell 转义

指定 `--parse-mode` 后，CLI 自动处理对应格式的转义，保留所选的 API 模式：`MarkdownV2` 仍发送 MarkdownV2，`HTML` 仍发送 HTML，`Markdown` 使用 Telegram 的旧版 Markdown 规则。不指定模式时，文本保持原样。

例如，下面两种调用都不需要手动处理 Telegram 转义：

```bash
tgpush message \
  --chat-id '@example_channel' \
  --parse-mode MarkdownV2 \
  --text '*版本 v1.2!* [说明](https://example.com/a(b)?x=1&y=2)'

tgpush message \
  --chat-id '@example_channel' \
  --parse-mode HTML \
  --text '<b>A & B</b> <a href="https://example.com/?x=1&y=2">详情</a>'
```

- MarkdownV2：保留成对的粗体、斜体、下划线、删除线、剧透、代码、链接和行首引用标记；根据正文、代码、链接地址的不同规则补齐转义。也接受 `**粗体**` 和 `~~删除线~~`，转换成 Telegram 对应标记。`__文本__` 按 Telegram 规则表示下划线。
- HTML：保留 Telegram 支持的成对标签，转义正文和属性值中的特殊字符；已有的 `&lt;`、`&gt;`、`&amp;`、`&quot;` 和有效数字实体不会重复转义。数字实体会按 Telegram 解析器的限制规范化；属性先解码实体再校验和转义，保留链接地址的实际含义。
- 未闭合的格式标记、不支持的 HTML 标签和命名实体按普通文字显示。代码内的 HTML 标签按代码文字处理。普通 Markdown 图片显示为链接；自定义表情仍使用 Telegram 的专用语法。
- 消息正文和所有媒体标题共用此逻辑，适用于命令行参数、UTF-8 文件和标准输入。显式提供 `entities` / `caption_entities` 时不改写文本，保留实体偏移。

行内代码只匹配同一行内等长的反引号，多行代码请使用围栏代码块。未配对的反引号按文字保留。HTML 链接地址中需要保留为文字的实体形式，通过对其中一个非保留字母或数字进行百分号编码，避免 Telegram 再次解码（例如 `&lt;` 转为 `&l%74;`）；URL 分隔符保持不变。

这里处理的是 Telegram 格式转义，不会把字面量 `\n`、`\t` 解释成换行或制表符。已转义的格式特殊字符会保留，其他反斜杠在 MarkdownV2 中按字面量处理。shell 引号和 JSON 语法仍遵循各自规则。

规则来源：[Telegram Bot API 格式说明](https://core.telegram.org/bots/api#formatting-options)、[MarkdownV2](https://core.telegram.org/bots/api#markdownv2-style)、[HTML](https://core.telegram.org/bots/api#html-style)。

以下 Bash 调用向 Telegram 发送的是字面量 `\n`，而不是换行：

```bash
tgpush message \
  --chat-id '@example_channel' \
  --text 'line one\nline two'
```

如需真实换行，使用 shell 的多行字符串或文本文件：

```bash
tgpush message \
  --chat-id '@example_channel' \
  --text-file ./message.txt
```

也可以从标准输入读取：

```bash
printf '第一行\n第二行\n' | tgpush message \
  --chat-id '@example_channel' \
  --text-file -
```

PowerShell 示例：

```powershell
tgpush message `
  --chat-id '@example_channel' `
  --text-file .\message.txt
```

复杂参数建议使用 JSON 文件，避免 shell 转义：

```json
{
  "inline_keyboard": [
    [
      {
        "text": "打开网站",
        "url": "https://example.com"
      }
    ]
  ]
}
```

```bash
tgpush message \
  --chat-id '@example_channel' \
  --text '请选择：' \
  --reply-markup-json-file ./keyboard.json
```

`--text-file`、`--caption-file` 和所有 `--*-json-file` 都接受 `-` 作为标准输入。一次调用只能有一个这样的参数使用 `-`。

## 输出、错误和退出码

Telegram Bot API 的响应是 JSON 对象。只要 `tgpush` 收到有效 Telegram JSON，它就把 HTTP 响应正文逐字节原样写到标准输出：不重新序列化、不美化、不添加前缀，也不添加额外换行。

成功示例：

```json
{"ok":true,"result":{"message_id":42,"chat":{"id":-1001234567890}}}
```

Telegram API 错误也仍然写到标准输出：

```json
{"ok":false,"error_code":400,"description":"Bad Request: chat not found"}
```

因此调用方应始终读取 stdout，并检查 `ok`、`result`、`description`、`error_code` 和 `parameters`。例如：

```bash
tgpush message --chat-id '@example_channel' --text 'hello' | jq '.result.message_id'
```

退出码：

| 场景 | stdout | stderr | 退出码 |
|---|---|---|---:|
| Telegram 返回 `ok: true` | 原始 JSON | 空 | `0` |
| Telegram 返回 `ok: false` | 原始 JSON | 空 | `1` |
| HTTP 错误但正文仍是 Telegram JSON | 原始 JSON | 空 | `1` |
| 网络、TLS、超时、非 JSON 网关响应 | 空 | 本地诊断 | `1` |
| 参数、环境变量、本地文件或输入 JSON 错误 | 空 | 本地诊断 | `2` |
| `--help`、`--version` | 帮助或版本信息 | 空 | `0` |

本地诊断不会输出 Token 或包含 Token 的请求 URL。默认 Telegram API 不会在 JSON 响应中回显 Token；若配置了不可信的自建 API 地址，服务端返回的 JSON 仍会按原样写出。

## Telegram 限制说明

Bot API 的限制由 Telegram 决定，可能随版本变化。提供的 API 文档中列出的重要限制包括：

- 云端 Bot API 通过 URL 发送图片通常限制为 5 MB，其他内容通常限制为 20 MB。
- 本地上传图片通常限制为 10 MB，其他媒体通常限制为 50 MB。
- `sendDocument` 通过 URL 发送时目前只保证 PDF 和 ZIP 可用。
- 通过 URL 发送语音的限制严于本地上传；使用本地 OGG/OPUS、MP3 或 M4A 更可靠。
- 自建 Telegram Bot API Server 可能允许更大的本地上传上限。

`tgpush` 不在客户端硬编码这些大小和媒体格式限制，以免阻碍自建 Bot API Server 或未来 API 更新；服务器会返回最终的 JSON 结果。

## 从源码构建

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked --release
```

可执行文件输出位置：

```text
target/release/tgpush
```

Windows 下为 `target/release/tgpush.exe`。

## 许可证

本项目使用 [MIT License](LICENSE)。完整版权信息见 [NOTICE](NOTICE)。
