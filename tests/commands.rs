use assert_cmd::Command;
use serde_json::{Map, Value, json};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn tgpush(server: &MockServer) -> Command {
    let mut command = Command::cargo_bin("tgpush").expect("tgpush binary should be available");
    command
        .env("TELEGRAM_BOT_TOKEN", "test-token")
        .env("TELEGRAM_BOT_API_BASE_URL", server.uri());
    command
}

fn success_response() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(b"{\"ok\":true,\"result\":{}}", "application/json")
}

#[tokio::test]
async fn forward_message_serializes_its_documented_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/forwardMessage"))
        .and(body_json(json!({
            "chat_id": "-1001",
            "message_thread_id": 7,
            "direct_messages_topic_id": 8,
            "from_chat_id": "@source",
            "message_id": 12,
            "video_start_timestamp": 9,
            "disable_notification": true,
            "protect_content": true,
            "message_effect_id": "effect",
            "suggested_post_parameters": {"price": 1}
        })))
        .respond_with(success_response())
        .mount(&server)
        .await;

    let output = tgpush(&server)
        .args([
            "forward-message",
            "--chat-id",
            "-1001",
            "--message-thread-id",
            "7",
            "--direct-messages-topic-id",
            "8",
            "--from-chat-id",
            "@source",
            "--message-id",
            "12",
            "--video-start-timestamp",
            "9",
            "--disable-notification",
            "--protect-content",
            "--message-effect-id",
            "effect",
            "--suggested-post-parameters-json",
            "{\"price\":1}",
        ])
        .output()
        .expect("run tgpush");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"{\"ok\":true,\"result\":{}}");
}

#[tokio::test]
async fn forward_messages_serializes_repeated_message_ids_as_an_array() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/forwardMessages"))
        .and(body_json(json!({
            "chat_id": "@target",
            "from_chat_id": "-1002",
            "message_ids": [3, 4, 10],
            "disable_notification": true,
            "protect_content": true
        })))
        .respond_with(success_response())
        .mount(&server)
        .await;

    let output = tgpush(&server)
        .args([
            "forward-messages",
            "--chat-id",
            "@target",
            "--from-chat-id",
            "-1002",
            "--message-id",
            "3",
            "--message-id",
            "4",
            "--message-id",
            "10",
            "--disable-notification",
            "--protect-content",
        ])
        .output()
        .expect("run tgpush");

    assert_eq!(output.status.code(), Some(0));
}

#[tokio::test]
async fn all_remote_media_commands_use_their_matching_api_method() {
    let commands = [
        ("photo", "photo", "sendPhoto"),
        ("audio", "audio", "sendAudio"),
        ("document", "document", "sendDocument"),
        ("video", "video", "sendVideo"),
        ("animation", "animation", "sendAnimation"),
        ("voice", "voice", "sendVoice"),
    ];

    for (command_name, media_field, method_name) in commands {
        let server = MockServer::start().await;
        let mut expected = Map::new();
        expected.insert("chat_id".to_owned(), Value::String("@target".to_owned()));
        expected.insert(
            media_field.to_owned(),
            Value::String("test-file-id".to_owned()),
        );
        Mock::given(method("POST"))
            .and(path(format!("/bottest-token/{method_name}")))
            .and(body_json(Value::Object(expected)))
            .respond_with(success_response())
            .mount(&server)
            .await;

        let output = tgpush(&server)
            .args([
                command_name,
                "--chat-id",
                "@target",
                &format!("--{media_field}"),
                "test-file-id",
            ])
            .output()
            .expect("run tgpush");

        assert_eq!(
            output.status.code(),
            Some(0),
            "{command_name} should use {method_name}"
        );
    }
}

#[tokio::test]
async fn video_serializes_common_caption_and_media_options() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/bottest-token/sendVideo"))
        .and(body_json(json!({
            "chat_id": "@target",
            "business_connection_id": "business",
            "receiver_user_id": 10,
            "callback_query_id": "callback",
            "disable_notification": true,
            "protect_content": true,
            "allow_paid_broadcast": true,
            "message_effect_id": "effect",
            "suggested_post_parameters": {"price": 1},
            "reply_parameters": {"message_id": 5},
            "reply_markup": {"inline_keyboard": []},
            "caption": "caption",
            "parse_mode": "HTML",
            "video": "video-file-id",
            "duration": 12,
            "width": 640,
            "height": 360,
            "cover": "cover-file-id",
            "start_timestamp": 4,
            "show_caption_above_media": true,
            "has_spoiler": true,
            "supports_streaming": true
        })))
        .respond_with(success_response())
        .mount(&server)
        .await;

    let output = tgpush(&server)
        .args([
            "video",
            "--chat-id",
            "@target",
            "--business-connection-id",
            "business",
            "--receiver-user-id",
            "10",
            "--callback-query-id",
            "callback",
            "--disable-notification",
            "--protect-content",
            "--allow-paid-broadcast",
            "--message-effect-id",
            "effect",
            "--suggested-post-parameters-json",
            "{\"price\":1}",
            "--reply-parameters-json",
            "{\"message_id\":5}",
            "--reply-markup-json",
            "{\"inline_keyboard\":[]}",
            "--caption",
            "caption",
            "--parse-mode",
            "HTML",
            "--video",
            "video-file-id",
            "--duration",
            "12",
            "--width",
            "640",
            "--height",
            "360",
            "--cover",
            "cover-file-id",
            "--start-timestamp",
            "4",
            "--show-caption-above-media",
            "--has-spoiler",
            "--supports-streaming",
        ])
        .output()
        .expect("run tgpush");

    assert_eq!(output.status.code(), Some(0));
}
