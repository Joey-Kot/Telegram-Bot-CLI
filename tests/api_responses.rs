use assert_cmd::Command;
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
async fn check_preserves_success_json_byte_for_byte() {
    let server = MockServer::start().await;
    let body = b"{\n  \"ok\": true,\n  \"result\": {\"id\": 42, \"username\": \"bot\"}\n}\n";
    Mock::given(method("POST"))
        .and(path("/bottest-token/getMe"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .mount(&server)
        .await;

    let output = tgpush(&server).arg("check").output().expect("run tgpush");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, body);
    assert!(output.stderr.is_empty());
}

#[tokio::test]
async fn check_preserves_error_json_byte_for_byte_on_stdout() {
    let server = MockServer::start().await;
    let body = b"{ \"ok\":false, \"error_code\":401, \"description\":\"Unauthorized\" }";
    Mock::given(method("POST"))
        .and(path("/bottest-token/getMe"))
        .respond_with(ResponseTemplate::new(401).set_body_raw(body, "application/json"))
        .mount(&server)
        .await;

    let output = tgpush(&server).arg("check").output().expect("run tgpush");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, body);
    assert!(output.stderr.is_empty());
}

#[tokio::test]
async fn http_error_with_ok_true_still_exits_with_failure() {
    let server = MockServer::start().await;
    let body = b"{\"ok\":true,\"result\":{\"unexpected\":true}}";
    Mock::given(method("POST"))
        .and(path("/bottest-token/getMe"))
        .respond_with(ResponseTemplate::new(500).set_body_raw(body, "application/json"))
        .mount(&server)
        .await;

    let output = tgpush(&server).arg("check").output().expect("run tgpush");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, body);
    assert!(output.stderr.is_empty());
}

#[tokio::test]
async fn non_json_responses_do_not_write_to_stdout_or_expose_the_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/getMe"))
        .respond_with(ResponseTemplate::new(502).set_body_string("<html>bad gateway</html>"))
        .mount(&server)
        .await;

    let output = tgpush(&server).arg("check").output().expect("run tgpush");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("不是有效 JSON"));
    assert!(!stderr.contains("test-token"));
}

#[tokio::test]
async fn message_uses_json_and_keeps_literal_backslash_sequences() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendMessage"))
        .and(body_json(json!({
            "chat_id": "@example_channel",
            "text": "literal \\\\n, emoji 😀",
            "parse_mode": "MarkdownV2"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            b"{\"ok\":true,\"result\":{\"message_id\":99}}",
            "application/json",
        ))
        .mount(&server)
        .await;

    let output = tgpush(&server)
        .args([
            "message",
            "--chat-id",
            "@example_channel",
            "--text",
            "literal \\n, emoji 😀",
            "--parse-mode",
            "MarkdownV2",
        ])
        .output()
        .expect("run tgpush");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        b"{\"ok\":true,\"result\":{\"message_id\":99}}"
    );
    assert!(output.stderr.is_empty());
}

#[tokio::test]
async fn photo_upload_uses_multipart_and_preserves_the_file() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendPhoto"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            b"{\"ok\":true,\"result\":{\"message_id\":7}}",
            "application/json",
        ))
        .mount(&server)
        .await;

    let file = NamedTempFile::new().expect("create temp image");
    std::fs::write(file.path(), b"fake-image-bytes").expect("write temp image");

    let output = tgpush(&server)
        .args([
            "photo",
            "--chat-id",
            "-100123",
            "--photo-file",
            file.path().to_str().expect("temp path should be utf-8"),
            "--caption",
            "图片",
        ])
        .output()
        .expect("run tgpush");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        b"{\"ok\":true,\"result\":{\"message_id\":7}}"
    );

    let requests = server
        .received_requests()
        .await
        .expect("requests are available");
    assert_eq!(requests.len(), 1);
    let body = String::from_utf8_lossy(&requests[0].body);
    assert!(body.contains("name=\"chat_id\""));
    assert!(body.contains("-100123"));
    assert!(body.contains("name=\"caption\""));
    assert!(body.contains("图片"));
    assert!(body.contains("name=\"photo\""));
    assert!(body.contains("fake-image-bytes"));
}
