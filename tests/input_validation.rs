use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use tempfile::NamedTempFile;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn tgpush(server: &MockServer) -> Command {
    let mut command = Command::cargo_bin("tgpush").expect("tgpush binary should be available");
    command
        .env("TELEGRAM_BOT_TOKEN", "test-token")
        .env("TELEGRAM_BOT_API_BASE_URL", server.uri());
    command
}

#[tokio::test]
async fn text_and_json_files_are_read_as_utf8_without_escape_rewriting() {
    let server = MockServer::start().await;
    let text_file = NamedTempFile::new().expect("create text file");
    std::fs::write(text_file.path(), "第一行\nliteral \\n\n").expect("write text");
    let keyboard_file = NamedTempFile::new().expect("create JSON file");
    std::fs::write(keyboard_file.path(), "{\"inline_keyboard\":[]}").expect("write JSON");

    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .and(body_json(json!({
            "chat_id": "@target",
            "text": "第一行\nliteral \\n\n",
            "reply_markup": {"inline_keyboard": []}
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(b"{\"ok\":true,\"result\":{}}", "application/json"),
        )
        .mount(&server)
        .await;

    tgpush(&server)
        .args([
            "message",
            "--chat-id",
            "@target",
            "--text-file",
            text_file.path().to_str().expect("UTF-8 temp path"),
            "--reply-markup-json-file",
            keyboard_file.path().to_str().expect("UTF-8 temp path"),
        ])
        .assert()
        .success()
        .stdout("{\"ok\":true,\"result\":{}}");
}

#[tokio::test]
async fn text_file_dash_reads_stdin_once_and_preserves_real_newlines() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .and(body_json(json!({
            "chat_id": "@target",
            "text": "第一行\n第二行\n"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(b"{\"ok\":true,\"result\":{}}", "application/json"),
        )
        .mount(&server)
        .await;

    let mut command = tgpush(&server);
    let output = command
        .args(["message", "--chat-id", "@target", "--text-file", "-"])
        .write_stdin("第一行\n第二行\n")
        .output()
        .expect("run tgpush");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"{\"ok\":true,\"result\":{}}");
}

#[test]
fn parse_mode_and_entities_are_rejected_locally() {
    let mut command = Command::cargo_bin("tgpush").expect("tgpush binary should be available");
    command
        .args([
            "message",
            "--chat-id",
            "@target",
            "--text",
            "hello",
            "--parse-mode",
            "HTML",
            "--entities-json",
            "[]",
        ])
        .env("TELEGRAM_BOT_TOKEN", "test-token")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("不能与"));
}

#[test]
fn a_thumbnail_requires_a_local_main_media_file() {
    let thumbnail = NamedTempFile::new().expect("create thumbnail");
    std::fs::write(thumbnail.path(), b"thumbnail").expect("write thumbnail");

    let mut command = Command::cargo_bin("tgpush").expect("tgpush binary should be available");
    command
        .args([
            "audio",
            "--chat-id",
            "@target",
            "--audio",
            "remote-file-id",
            "--thumbnail-file",
            thumbnail.path().to_str().expect("UTF-8 temp path"),
        ])
        .env("TELEGRAM_BOT_TOKEN", "test-token")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("--thumbnail-file"));
}
