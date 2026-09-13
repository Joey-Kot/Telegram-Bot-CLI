const RESERVED: &str = "_*[]()~`>#+-=|{}.!\\";
const LEGACY_RESERVED: &str = "_*`[";

#[derive(Debug)]
enum Token<'a> {
    Text(&'a str),
    Marker {
        source: &'a str,
        paired: bool,
    },
    Code {
        body: &'a str,
        language: &'a str,
        block: bool,
    },
    Link {
        label: &'a str,
        url: &'a str,
        image: bool,
    },
    Syntax(&'a str),
}

pub(super) fn prepare(text: &str, legacy: bool) -> String {
    render(text, legacy, true)
}

fn escaped(text: &str, reserved: &str) -> String {
    escaped_with(text, reserved, reserved)
}

fn escaped_with(text: &str, reserved: &str, recognized: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek().is_some_and(|next| recognized.contains(*next)) {
            out.push(ch);
            out.push(chars.next().expect("peeked character"));
        } else {
            if reserved.contains(ch) {
                out.push('\\');
            }
            out.push(ch);
        }
    }
    out
}

// Skip escaped characters when searching for syntax, including escaped closers.
fn closing(text: &str, start: usize, delimiter: &str) -> Option<usize> {
    let mut at = start;
    while at < text.len() {
        if text[at..].starts_with(delimiter) {
            return Some(at);
        }
        let ch = text[at..].chars().next()?;
        at += ch.len_utf8();
        if ch == '\\' {
            at += text[at..].chars().next().map_or(0, char::len_utf8);
        }
    }
    None
}

fn balanced(text: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 1usize;
    let mut chars = text[start..].char_indices();
    while let Some((offset, ch)) = chars.next() {
        if ch == '\\' {
            if let Some((_, escaped)) = chars.next()
                && close == ')'
                && escaped == ')'
                && depth > 1
            {
                depth -= 1;
            }
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(start + offset);
            }
        }
    }
    None
}

fn tokenize(text: &str, legacy: bool, links: bool) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut stack: Vec<(&str, usize)> = Vec::new();
    let mut at = 0;
    let mut quoted_line = false;
    while at < text.len() {
        let rest = &text[at..];
        let ch = rest.chars().next().expect("nonempty suffix");
        let line_start = at == 0 || text.as_bytes()[at - 1] == b'\n';
        if !legacy && links && line_start {
            quoted_line = rest.starts_with('>') || rest.starts_with("**>");
            if quoted_line {
                let size = if rest.starts_with("**>") { 3 } else { 1 };
                tokens.push(Token::Syntax(&rest[..size]));
                at += size;
                continue;
            }
        }
        if ch == '\\' {
            let end = at + 1 + rest[1..].chars().next().map_or(0, char::len_utf8);
            tokens.push(Token::Text(&text[at..end]));
            at = end;
            continue;
        }
        if !legacy
            && quoted_line
            && rest.starts_with("||")
            && (rest.len() == 2 || rest[2..].starts_with('\n'))
            && !stack.iter().any(|(marker, _)| *marker == "||")
        {
            tokens.push(Token::Syntax("||"));
            at += 2;
            continue;
        }
        if ch == '`' {
            let size = rest.bytes().take_while(|b| *b == b'`').count();
            let delimiter = &rest[..size];
            if let Some(end) = closing(text, at + size, delimiter) {
                let mut body = &text[at + size..end];
                let mut language = "";
                let block = size >= 3;
                if block
                    && let Some((info, content)) = body.split_once('\n')
                    && info
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "_+-#".contains(c))
                {
                    language = info;
                    body = content;
                }
                tokens.push(Token::Code {
                    body,
                    language,
                    block,
                });
                at = end + size;
                continue;
            }
            tokens.push(Token::Text(delimiter));
            at += size;
            continue;
        }
        if !legacy
            && rest.starts_with("**_")
            && at > 0
            && text.as_bytes()[at - 1] == b'_'
            && !stack.last().is_some_and(|s| s.0 == "**")
        {
            tokens.push(Token::Syntax("**"));
            at += 2;
            continue;
        }
        let image = !legacy && rest.starts_with("![");
        if links && (ch == '[' || image) {
            let label_start = at + if image { 2 } else { 1 };
            if let Some(label_end) = balanced(text, label_start, '[', ']')
                && text[label_end..].starts_with("](")
                && let Some(url_end) = balanced(text, label_end + 2, '(', ')')
            {
                tokens.push(Token::Link {
                    label: &text[label_start..label_end],
                    url: &text[label_end + 2..url_end],
                    image,
                });
                at = url_end + 1;
                continue;
            }
        }
        // Telegram uses __ for underline. Resolve a closing ___ from the
        // innermost delimiter outwards, then serialize with an empty-bold separator.
        let marker = if let Some((top, _)) = stack.last()
            && matches!(*top, "*" | "~")
            && rest.starts_with(top)
        {
            Some(*top)
        } else if !legacy && rest.starts_with("___") && stack.last().is_some_and(|s| s.0 == "_") {
            Some("_")
        } else {
            ["**", "__", "~~", "||", "*", "_", "~"]
                .into_iter()
                .find(|m| rest.starts_with(m) && (!legacy || matches!(*m, "**" | "*" | "_")))
        };
        if let Some(marker) = marker {
            let index = tokens.len();
            let paired = stack.last().is_some_and(|s| s.0 == marker);
            if paired {
                let (_, opening) = stack.pop().expect("matching opener");
                if let Token::Marker { paired, .. } = &mut tokens[opening] {
                    *paired = true;
                }
            } else {
                stack.push((marker, index));
            }
            tokens.push(Token::Marker {
                source: &rest[..marker.len()],
                paired,
            });
            at += marker.len();
        } else {
            tokens.push(Token::Text(&rest[..ch.len_utf8()]));
            at += ch.len_utf8();
        }
    }
    tokens
}

fn marker(source: &str) -> &str {
    match source {
        "**" => "*",
        "~~" => "~",
        _ => source,
    }
}

#[derive(Default)]
struct Writer<'a> {
    out: String,
    opened: Vec<&'a str>,
    legacy: bool,
}

impl<'a> Writer<'a> {
    fn delimiter(&mut self, delimiter: &str) {
        if !self.legacy && self.out.ends_with('_') && delimiter.starts_with('_') {
            self.out.push_str("**");
        }
        self.out.push_str(delimiter);
    }

    fn sync(&mut self, wanted: &[&'a str]) {
        let shared = self
            .opened
            .iter()
            .zip(wanted)
            .take_while(|(a, b)| a == b)
            .count();
        while self.opened.len() > shared {
            let last = self.opened.pop().expect("unshared marker");
            self.delimiter(last);
        }
        for item in &wanted[shared..] {
            self.delimiter(item);
            self.opened.push(item);
        }
    }
}

fn render(text: &str, legacy: bool, links: bool) -> String {
    let mut writer = Writer {
        legacy,
        ..Writer::default()
    };
    let mut active: Vec<&str> = Vec::new();
    let reserved = if legacy { LEGACY_RESERVED } else { RESERVED };
    for token in tokenize(text, legacy, links) {
        match token {
            Token::Marker {
                source,
                paired: true,
            } => {
                if active.last() == Some(&source) {
                    active.pop();
                } else {
                    active.push(source);
                }
            }
            Token::Text(value)
            | Token::Marker {
                source: value,
                paired: false,
            } => {
                if value == "\n" {
                    writer.sync(&[]);
                    writer.out.push('\n');
                    continue;
                }
                let value = escaped(value, reserved);
                // Legacy Markdown forbids escaping inside entities and nesting.
                let mut wanted: Vec<&str> = active.iter().map(|source| marker(source)).collect();
                wanted.dedup();
                if legacy {
                    if value.starts_with('\\') {
                        wanted.clear();
                    } else if let Some(last) = wanted.last().copied() {
                        wanted = vec![last];
                    }
                }
                writer.sync(&wanted);
                writer.out.push_str(&value);
            }
            Token::Code {
                body,
                language,
                block,
            } => {
                writer.sync(&[]);
                if !links || (legacy && body.contains('`')) {
                    writer.out.push_str(&escaped(body, reserved));
                    continue;
                }
                let delimiter = if block { "```" } else { "`" };
                writer.out.push_str(delimiter);
                if block {
                    writer.out.push_str(language);
                    writer.out.push('\n');
                }
                writer.out.push_str(&if legacy {
                    body.to_owned()
                } else {
                    escaped(body, "`\\")
                });
                writer.out.push_str(delimiter);
            }
            Token::Link { label, url, image } => {
                let wanted: Vec<&str> = if legacy {
                    vec![]
                } else {
                    active.iter().map(|source| marker(source)).collect()
                };
                writer.sync(&wanted);
                if image && (url.starts_with("tg://emoji?") || url.starts_with("tg://time?")) {
                    writer.out.push('!');
                }
                writer.out.push('[');
                writer.out.push_str(&render(label, legacy, false));
                writer.out.push_str("](");
                if legacy {
                    writer.out.push_str(
                        &url.replace("\\)", ")")
                            .replace('\\', "%5C")
                            .replace(')', "%29"),
                    );
                } else {
                    writer.out.push_str(&escaped_with(url, ")\\", RESERVED));
                }
                writer.out.push(')');
            }
            Token::Syntax(value) => {
                if value == "**" {
                    // This separator has no visible content; Writer inserts it
                    // again only where the normalized underscore nesting needs it.
                    continue;
                }
                writer.sync(&[]);
                writer.out.push_str(value);
            }
        }
    }
    writer.sync(&[]);
    writer.out
}

#[cfg(test)]
mod tests {
    use super::prepare;

    #[test]
    fn markdown_v2_contexts_and_idempotence() {
        let cases = [
            ("*a**b*", "*ab*"),
            ("~a~~b~", "~ab~"),
            (">q\n\\n ||", ">q\n\\\\n \\|\\|"),
            (
                r"[x](https://example.com/a\(b)",
                r"[x](https://example.com/a\(b)",
            ),
            (
                "版本 v1.2! a+b=c (x) #tag - item {ok}",
                r"版本 v1\.2\! a\+b\=c \(x\) \#tag \- item \{ok\}",
            ),
            (
                "*bold!* _italic._ __under__ ~gone~ ||secret!||",
                r"*bold\!* _italic\._ __under__ ~gone~ ||secret\!||",
            ),
            ("**bold!** and ~~gone.~~", r"*bold\!* and ~gone\.~"),
            ("*bold _italic!_ end*", r"*bold _italic\!_ end*"),
            ("___both!___", r"__**_both\!_**__"),
            (
                r"already \*literal\* and \. and \\ slash",
                r"already \*literal\* and \. and \\ slash",
            ),
            (r"literal \n and \path \", r"literal \\n and \\path \\"),
            ("`a_b * c < d! C:\\tmp`", "`a_b * c < d! C:\\\\tmp`"),
            (
                "```rust\nlet x = `a`; // C:\\tmp\n```",
                "```rust\nlet x = \\`a\\`; // C:\\\\tmp\n```",
            ),
            (
                "[read!](https://example.com/a(b)?x=1&y=2)",
                r"[read\!](https://example.com/a(b\)?x=1&y=2)",
            ),
            (
                r"[read\!](https://example.com/a(b\)?x=1&y=2)",
                r"[read\!](https://example.com/a(b\)?x=1&y=2)",
            ),
            (
                "[**read!**](tg://user?id=123)",
                r"[*read\!*](tg://user?id=123)",
            ),
            ("![😀](tg://emoji?id=123)", "![😀](tg://emoji?id=123)"),
            (
                "![image](https://example.com/a.png)",
                "[image](https://example.com/a.png)",
            ),
            (
                ">quote!\n>second.\n**>expand!||",
                ">quote\\!\n>second\\.\n**>expand\\!||",
            ),
            ("unclosed *bold and [link", r"unclosed \*bold and \[link"),
            ("*before `x_y` after*", "*before *`x_y`* after*"),
            (
                "# heading\n- list!\n1. item",
                "\\# heading\n\\- list\\!\n1\\. item",
            ),
        ];
        for (input, expected) in cases {
            let actual = prepare(input, false);
            assert_eq!(actual, expected, "input: {input:?}");
            assert_eq!(
                prepare(&actual, false),
                actual,
                "repeated conversion: {input:?}"
            );
        }
    }

    #[test]
    fn legacy_markdown_keeps_its_own_rules() {
        assert_eq!(
            prepare("*bold!* and _italic._ (x)", true),
            "*bold!* and _italic._ (x)"
        );
        assert_eq!(prepare(r"*a\*b*", true), r"*a*\**b*");
        assert_eq!(
            prepare("[x](https://example.com/a(b))", true),
            "[x](https://example.com/a(b%29)"
        );
    }

    #[test]
    fn mixed_punctuation_is_not_escaped_again() {
        let chunks = [
            "_", "__", "*", "**", "~", "||", "`", "\\_", "\\n", " ", "\n", ">", "中", "a", ".",
        ];
        let mut state = 42u64;
        for _ in 0..256 {
            let mut input = String::new();
            for _ in 0..24 {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                input.push_str(chunks[(state >> 32) as usize % chunks.len()]);
            }
            let once = prepare(&input, false);
            assert_eq!(prepare(&once, false), once, "input: {input:?}");
        }
    }
}
