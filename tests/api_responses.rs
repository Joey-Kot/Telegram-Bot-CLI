use assert_cmd::Command;
use wiremock::matchers::{method, path};
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
