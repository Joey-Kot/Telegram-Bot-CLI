use assert_cmd::Command;
use serde_json::json;
use tempfile::NamedTempFile;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn tgpush(server: &MockServer) -> Command {
    let mut command = Command::cargo_bin("tgpush").expect("tgpush binary");
    command
        .env("TELEGRAM_BOT_TOKEN", "test-token")
        .env("TELEGRAM_BOT_API_BASE_URL", server.uri());
    command
}

fn success() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({"ok": true, "result": {}}))
}

#[tokio::test]
async fn message_escapes_the_selected_format_without_changing_parse_mode() {
    let server = MockServer::start().await;
    for (mode, input, expected) in [
        (
            "MarkdownV2",
            "*版本 v1.2!* [说明](https://example.com/a(b)?x=1&y=2)",
            r"*版本 v1\.2\!* [说明](https://example.com/a(b\)?x=1&y=2)",
        ),
        (
            "HTML",
            "<b>A & B</b> <a href='https://example.com/?x=1&y=2'>链接</a>",
            r#"<b>A &amp; B</b> <a href="https://example.com/?x=1&amp;y=2">链接</a>"#,
        ),
        ("Markdown", "*bold!* [literal", r"*bold!* \[literal"),
    ] {
        Mock::given(method("POST"))
            .and(path("/bottest-token/sendMessage"))
            .and(body_json(json!({
                "chat_id": "@target", "text": expected, "parse_mode": mode
            })))
            .respond_with(success())
            .expect(1)
            .mount(&server)
            .await;
        tgpush(&server)
            .args([
                "message",
                "--chat-id",
                "@target",
                "--text",
                input,
                "--parse-mode",
                mode,
            ])
            .assert()
            .success();
    }
}

#[tokio::test]
async fn file_and_stdin_inputs_use_the_same_formatting_pipeline() {
    let server = MockServer::start().await;
    let file = NamedTempFile::new().expect("text file");
    std::fs::write(file.path(), "<b>文件 & 标准输入</b>\n1 < 2\n").expect("write text");
    for source in [file.path().to_str().expect("UTF-8 path"), "-"] {
        Mock::given(method("POST"))
            .and(path("/bottest-token/sendMessage"))
            .and(body_json(json!({
                "chat_id": "@target",
                "text": "<b>文件 &amp; 标准输入</b>\n1 &lt; 2\n",
                "parse_mode": "HTML"
            })))
            .respond_with(success())
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;
        tgpush(&server)
            .args([
                "message",
                "--chat-id",
                "@target",
                "--text-file",
                source,
                "--parse-mode",
                "HTML",
            ])
            .write_stdin("<b>文件 & 标准输入</b>\n1 < 2\n")
            .assert()
            .success();
    }
}

#[tokio::test]
async fn every_media_caption_uses_the_selected_format() {
    let server = MockServer::start().await;
    for (command, api) in [
        ("photo", "sendPhoto"),
        ("audio", "sendAudio"),
        ("document", "sendDocument"),
        ("video", "sendVideo"),
        ("animation", "sendAnimation"),
        ("voice", "sendVoice"),
    ] {
        for (mode, input, expected) in [
            ("MarkdownV2", "*图片 v1.2!*", r"*图片 v1\.2\!*"),
            ("HTML", "<b>图片 & 标题</b>", "<b>图片 &amp; 标题</b>"),
        ] {
            Mock::given(method("POST"))
                .and(path(format!("/bottest-token/{api}")))
                .and(body_json(json!({
                    "chat_id": "@target",
                    command: "file-id",
                    "caption": expected,
                    "parse_mode": mode
                })))
                .respond_with(success())
                .expect(1)
                .mount(&server)
                .await;
            tgpush(&server)
                .args([
                    command,
                    "--chat-id",
                    "@target",
                    &format!("--{command}"),
                    "file-id",
                    "--caption",
                    input,
                    "--parse-mode",
                    mode,
                ])
                .assert()
                .success();
        }
    }
}

#[tokio::test]
async fn uploaded_media_formats_caption_files_in_multipart() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendPhoto"))
        .respond_with(success())
        .expect(1)
        .mount(&server)
        .await;
    let photo = NamedTempFile::new().expect("photo");
    std::fs::write(photo.path(), b"photo-bytes").expect("photo bytes");
    let caption = NamedTempFile::new().expect("caption");
    std::fs::write(caption.path(), "*v1.2!* `C:\\tmp`\n").expect("caption text");
    tgpush(&server)
        .args([
            "photo",
            "--chat-id",
            "@target",
            "--photo-file",
            photo.path().to_str().expect("photo path"),
            "--caption-file",
            caption.path().to_str().expect("caption path"),
            "--parse-mode",
            "MarkdownV2",
        ])
        .assert()
        .success();
    let requests = server.received_requests().await.expect("requests");
    let body = String::from_utf8_lossy(&requests[0].body);
    assert!(body.contains("name=\"caption\"\r\n\r\n*v1\\.2\\!* `C:\\\\tmp`\n\r\n"));
    assert!(body.contains("name=\"parse_mode\"\r\n\r\nMarkdownV2\r\n"));
    assert!(body.contains("photo-bytes"));
}

#[tokio::test]
async fn explicit_entities_keep_the_original_text_and_utf16_offsets() {
    let server = MockServer::start().await;
    let input = "😀 *literal* & <tag>";
    let entities = json!([{"type": "bold", "offset": 3, "length": 9}]);
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .and(body_json(json!({
            "chat_id": "@target", "text": input, "entities": entities
        })))
        .respond_with(success())
        .expect(1)
        .mount(&server)
        .await;
    tgpush(&server)
        .args([
            "message",
            "--chat-id",
            "@target",
            "--text",
            input,
            "--entities-json",
            &entities.to_string(),
        ])
        .assert()
        .success();
}
