use assert_cmd::Command;
use predicates::prelude::*;

fn tgpush() -> Command {
    Command::cargo_bin("tgpush").expect("tgpush binary should be available")
}

#[test]
fn root_help_lists_the_public_commands() {
    tgpush()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("forward-messages"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("animation"))
        .stdout(predicate::str::contains("--proxy <URL>"))
        .stdout(predicate::str::contains("TELEGRAM_BOT_TOKEN"))
        .stdout(predicate::str::contains("123456789:AAExampleBotToken"))
        .stdout(predicate::str::contains("send-message").not());
}

#[test]
fn delete_requires_a_chat_and_one_positive_message_id() {
    for args in [
        vec!["delete", "--chat-id", "@target"],
        vec!["delete", "--message-id", "1"],
        vec!["delete", "--chat-id", "@target", "--message-id", "0"],
        vec!["delete", "--chat-id", "@target", "--message-id=-1"],
        vec!["delete", "--chat-id", "@target", "--message-id", "abc"],
        vec![
            "delete",
            "--chat-id",
            "@target",
            "--message-id",
            "1",
            "--message-id",
            "2",
        ],
        vec![
            "delete",
            "--chat-id",
            "@target",
            "--message-id",
            "1",
            "--message-thread-id",
            "2",
        ],
    ] {
        tgpush()
            .args(args)
            .env("TELEGRAM_BOT_TOKEN", "test-token")
            .assert()
            .code(2)
            .stdout(predicate::str::is_empty());
    }
}

#[test]
fn short_help_is_not_a_public_argument() {
    tgpush()
        .arg("-h")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unexpected argument '-h'"));
}

#[test]
fn an_option_name_cannot_be_consumed_as_chat_id() {
    tgpush()
        .args(["message", "--chat-id", "--text", "--text", "hello"])
        .env("TELEGRAM_BOT_TOKEN", "test-token")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--chat-id"));
}

#[test]
fn missing_token_is_a_local_configuration_error() {
    tgpush()
        .arg("check")
        .env_remove("TELEGRAM_BOT_TOKEN")
        .env_remove("TELEGRAM_BOT_API_BASE_URL")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("TELEGRAM_BOT_TOKEN"));
}

#[test]
fn forward_messages_rejects_non_increasing_ids_before_network_access() {
    tgpush()
        .args([
            "forward-messages",
            "--chat-id",
            "-100123",
            "--from-chat-id",
            "-100456",
            "--message-id",
            "2",
            "--message-id",
            "1",
        ])
        .env("TELEGRAM_BOT_TOKEN", "test-token")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("严格递增"));
}
