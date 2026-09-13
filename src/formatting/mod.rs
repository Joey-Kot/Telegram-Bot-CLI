//! Telegram formatting is a wire format, not ordinary Markdown or browser HTML.
//! Escape text only after recognizing the surrounding syntax.
//! Rules: https://core.telegram.org/bots/api#formatting-options

mod html;
mod markdown;

pub fn prepare_text(text: &str, parse_mode: Option<&str>) -> String {
    match parse_mode {
        Some(mode) if mode.eq_ignore_ascii_case("MarkdownV2") => markdown::prepare(text, false),
        Some(mode) if mode.eq_ignore_ascii_case("Markdown") => markdown::prepare(text, true),
        Some(mode) if mode.eq_ignore_ascii_case("HTML") => html::prepare(text),
        _ => text.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::prepare_text;

    #[test]
    fn no_mode_preserves_every_byte() {
        let text = "**hi** <b>&test</b> \\n 😀\n";
        assert_eq!(prepare_text(text, None), text);
        assert_eq!(prepare_text(text, Some("future-mode")), text);
    }
}
