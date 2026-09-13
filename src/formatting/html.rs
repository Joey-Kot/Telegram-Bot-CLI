#[derive(Clone, Debug, PartialEq)]
struct Tag {
    name: String,
    opening: String,
}

enum Token<'a> {
    Text(&'a str),
    Open {
        raw: &'a str,
        tag: Tag,
        paired: bool,
    },
    Close {
        raw: &'a str,
        name: String,
        paired: bool,
    },
}

// Telegram accepts four named entities and numeric entities. Unknown or broken
// entities are literal text; escaping their ampersand prevents API parse errors.
fn entity(text: &str) -> Option<(usize, char)> {
    let end = text.find(';')?;
    let name = &text[1..end];
    let named = match name {
        "lt" => Some('<'),
        "gt" => Some('>'),
        "amp" => Some('&'),
        "quot" => Some('"'),
        _ => None,
    };
    if let Some(ch) = named {
        return Some((end + 1, ch));
    }
    let number = if let Some(hex) = name.strip_prefix("#x").or_else(|| name.strip_prefix("#X")) {
        if hex.is_empty() || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        u32::from_str_radix(hex, 16).ok()?
    } else {
        let decimal = name.strip_prefix('#')?;
        if decimal.is_empty() || !decimal.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        decimal.parse::<u32>().ok()?
    };
    if number == 0 {
        return None;
    }
    Some((end + 1, char::from_u32(number)?))
}

fn decode_attributes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        if rest.starts_with('&')
            && let Some((len, ch)) = entity(rest)
        {
            out.push(ch);
            at += len;
        } else {
            let ch = rest.chars().next().expect("nonempty suffix");
            out.push(ch);
            at += ch.len_utf8();
        }
    }
    out
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        if rest.starts_with('&')
            && let Some((len, ch)) = entity(rest)
        {
            // TDLib recognizes lowercase x only, and limits numeric references
            // to fewer than ten bytes before ';' and values below U+10FFFF.
            if rest.starts_with("&#") && (rest.starts_with("&#X") || len > 10 || ch == '\u{10ffff}')
            {
                if ch == '\u{10ffff}' {
                    out.push(ch);
                } else {
                    out.push_str(&format!("&#{};", ch as u32));
                }
            } else {
                out.push_str(&rest[..len]);
            }
            at += len;
            continue;
        }
        let ch = rest.chars().next().expect("nonempty suffix");
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
        at += ch.len_utf8();
    }
    out
}

// Attribute entities were already decoded before validating their meaning.
// Escape every ampersand here, including one followed by entity-looking text.
fn escape_attribute(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

// Telegram's live sendMessage path decodes entity-looking URL text once more
// after parsing HTML. Percent-encode an unreserved character in that text to
// prevent the extra decode. Keep URL separators (&, =, #, ;) intact; encoding
// the ampersand itself would change query parameter boundaries. This is also
// stable when preparing the same HTML more than once.
fn protect_url_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        if rest.starts_with('&')
            && let Some((len, _)) = entity(rest)
        {
            let last = len - 2; // last ASCII letter/digit before the semicolon
            out.push_str(&rest[..last]);
            out.push_str(&format!("%{:02X};", rest.as_bytes()[last]));
            at += len;
        } else {
            let ch = rest.chars().next().expect("nonempty suffix");
            out.push(ch);
            at += ch.len_utf8();
        }
    }
    out
}

fn tag_end(text: &str) -> Option<usize> {
    let mut quote = None;
    for (at, ch) in text.char_indices().skip(1) {
        if let Some(delimiter) = quote {
            if ch == delimiter {
                quote = None;
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ch == '>' {
            return Some(at + 1);
        } else if ch == '<' {
            return None;
        }
    }
    None
}

fn supported(name: &str) -> bool {
    matches!(
        name,
        "b" | "strong"
            | "i"
            | "em"
            | "u"
            | "ins"
            | "s"
            | "strike"
            | "del"
            | "span"
            | "tg-spoiler"
            | "a"
            | "code"
            | "pre"
            | "blockquote"
            | "tg-emoji"
            | "tg-time"
    )
}

fn attributes(mut text: &str) -> Option<Vec<(String, String)>> {
    let mut attrs = Vec::new();
    while !text.trim().is_empty() {
        text = text.trim_start();
        let end = text
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(text.len());
        if end == 0 {
            return None;
        }
        let name = text[..end].to_ascii_lowercase();
        text = text[end..].trim_start();
        let mut value = "";
        if let Some(rest) = text.strip_prefix('=') {
            text = rest.trim_start();
            if let Some(delimiter) = text.chars().next().filter(|c| *c == '\'' || *c == '"') {
                let end = text[1..].find(delimiter)? + 1;
                value = &text[1..end];
                text = &text[end + 1..];
            } else {
                let end = text.find(char::is_whitespace).unwrap_or(text.len());
                value = &text[..end];
                text = &text[end..];
                if value.is_empty() {
                    return None;
                }
            }
        }
        if attrs.iter().any(|(key, _)| key == &name) {
            return None;
        }
        attrs.push((name, decode_attributes(value)));
    }
    Some(attrs)
}

fn opening(name: &str, rest: &str) -> Option<Tag> {
    let attrs = attributes(rest)?;
    let get = |key: &str| {
        attrs
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    };
    let mut out = format!("<{name}");
    let mut attribute = |key: &str, value: &str| {
        out.push(' ');
        out.push_str(key);
        out.push_str("=\"");
        out.push_str(&escape_attribute(value));
        out.push('"');
    };
    match name {
        "a" => {
            if let Some(href) = get("href") {
                attribute("href", &protect_url_entities(href));
            }
        }
        "span" => {
            if get("class")? != "tg-spoiler" {
                return None;
            }
            attribute("class", "tg-spoiler");
        }
        "code" => {
            if let Some(class) = get("class").filter(|c| c.starts_with("language-")) {
                attribute("class", class);
            }
        }
        "tg-emoji" => {
            let id = get("emoji-id")?;
            id.parse::<u64>().ok().filter(|id| *id > 0)?;
            attribute("emoji-id", id);
        }
        "tg-time" => {
            let unix = get("unix")?;
            unix.parse::<i64>().ok()?;
            attribute("unix", unix);
            if let Some(format) = get("format") {
                attribute("format", format);
            }
        }
        "blockquote" if get("expandable").is_some() => out.push_str(" expandable"),
        _ => {}
    }
    out.push('>');
    Some(Tag {
        name: name.to_owned(),
        opening: out,
    })
}

fn tokenize(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut stack: Vec<(String, usize)> = Vec::new();
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        if rest.starts_with('<')
            && let Some(len) = tag_end(rest)
        {
            let raw = &rest[..len];
            let inner = &raw[1..len - 1];
            let close = inner.starts_with('/');
            let inner = inner.strip_prefix('/').unwrap_or(inner);
            let end = inner
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                .unwrap_or(inner.len());
            let name = inner[..end].to_ascii_lowercase();
            let attrs = &inner[end..];
            let in_code = stack
                .iter()
                .any(|(name, _)| matches!(name.as_str(), "code" | "pre"));
            let code_boundary = close && matches!(name.as_str(), "code" | "pre")
                || !close && name == "code" && stack.last().is_some_and(|(name, _)| name == "pre");
            if supported(&name) && (!in_code || code_boundary) {
                if close && attrs.trim().is_empty() {
                    let paired =
                        if let Some(position) = stack.iter().rposition(|(key, _)| *key == name) {
                            let opening = stack[position].1;
                            stack.truncate(position);
                            if let Token::Open { paired, .. } = &mut tokens[opening] {
                                *paired = true;
                            }
                            true
                        } else {
                            false
                        };
                    tokens.push(Token::Close { raw, name, paired });
                    at += len;
                    continue;
                } else if !close && let Some(tag) = opening(&name, attrs) {
                    stack.push((name, tokens.len()));
                    tokens.push(Token::Open {
                        raw,
                        tag,
                        paired: false,
                    });
                    at += len;
                    continue;
                }
            }
            tokens.push(Token::Text(raw));
            at += len;
        } else {
            let len = rest
                .as_bytes()
                .iter()
                .skip(1)
                .position(|b| *b == b'<')
                .map_or(rest.len(), |i| i + 1);
            tokens.push(Token::Text(&rest[..len]));
            at += len;
        }
    }
    tokens
}

#[derive(Default)]
struct Writer {
    out: String,
    opened: Vec<Tag>,
}

impl Writer {
    fn sync(&mut self, wanted: &[Tag]) {
        let shared = self
            .opened
            .iter()
            .zip(wanted)
            .take_while(|(a, b)| a == b)
            .count();
        while self.opened.len() > shared {
            let tag = self.opened.pop().expect("unshared tag");
            self.out.push_str(&format!("</{}>", tag.name));
        }
        for tag in &wanted[shared..] {
            self.out.push_str(&tag.opening);
            self.opened.push(tag.clone());
        }
    }
}

// Code/pre cannot overlap styles; blockquotes cannot nest. Close and reopen
// surrounding styles around code, instead of generating forbidden entities.
fn allowed(active: &[Tag]) -> Vec<Tag> {
    let code = active
        .iter()
        .any(|tag| matches!(tag.name.as_str(), "code" | "pre"));
    let mut result: Vec<Tag> = Vec::new();
    for tag in active {
        if code && !matches!(tag.name.as_str(), "code" | "pre") {
            continue;
        }
        if result.iter().any(|prior| prior.name == tag.name) {
            continue;
        }
        if code && tag.name == "pre" {
            result.insert(0, tag.clone());
        } else {
            result.push(tag.clone());
        }
    }
    result
}

pub(super) fn prepare(text: &str) -> String {
    let mut active = Vec::new();
    let mut writer = Writer::default();
    for token in tokenize(text) {
        match token {
            Token::Open {
                tag, paired: true, ..
            } => active.push(tag),
            Token::Close {
                name, paired: true, ..
            } => {
                if let Some(position) = active.iter().rposition(|tag| tag.name == name) {
                    active.truncate(position);
                }
            }
            Token::Text(value)
            | Token::Open {
                raw: value,
                paired: false,
                ..
            }
            | Token::Close {
                raw: value,
                paired: false,
                ..
            } => {
                writer.sync(&allowed(&active));
                writer.out.push_str(&escape(value));
            }
        }
    }
    writer.sync(&[]);
    if writer.out.is_empty() && !text.is_empty() {
        escape(text)
    } else {
        writer.out
    }
}

#[cfg(test)]
mod tests {
    use super::prepare;

    #[test]
    fn href_protection_preserves_url_separators_and_is_idempotent() {
        let input =
            "<a href='https://example.com/&amp;lt;?a=1&amp;lt;=2&amp;b=&amp;#60;#&amp;quot;'>x</a>";
        let expected = "<a href=\"https://example.com/&amp;l%74;?a=1&amp;l%74;=2&amp;b=&amp;#6%30;#&amp;quo%74;\">x</a>";
        assert_eq!(prepare(input), expected);
        assert_eq!(prepare(expected), expected);
        assert_eq!(
            prepare("<a href='https://example.com/?a=1&b=2&x=&lt;'>x</a>"),
            "<a href=\"https://example.com/?a=1&amp;b=2&amp;x=&lt;\">x</a>"
        );
    }

    #[test]
    fn html_contexts_and_idempotence() {
        let cases = [
            (
                "<a href='https://example.com/?x=&amp;lt;'>x</a>",
                "<a href=\"https://example.com/?x=&amp;l%74;\">x</a>",
            ),
            ("&#X41; &#00000065;", "&#65; &#65;"),
            (
                "<span class='tg&#45;spoiler'>x</span>",
                "<span class=\"tg-spoiler\">x</span>",
            ),
            (
                "<tg-emoji emoji-id='&#49;23'>😀</tg-emoji>",
                "<tg-emoji emoji-id=\"123\">😀</tg-emoji>",
            ),
            ("<a>https://example.com</a>", "<a>https://example.com</a>"),
            ("<b></b>", "&lt;b&gt;&lt;/b&gt;"),
            ("中文 😀 1 < 2 & 3 > 2", "中文 😀 1 &lt; 2 &amp; 3 &gt; 2"),
            (
                "<b>Tom & Jerry</b> <i>a < b</i>",
                "<b>Tom &amp; Jerry</b> <i>a &lt; b</i>",
            ),
            (
                "<b>bold <i>italic & more</i></b>",
                "<b>bold <i>italic &amp; more</i></b>",
            ),
            (
                "<a href='https://example.com/?a=1&b=\"two\"'>A&B</a>",
                "<a href=\"https://example.com/?a=1&amp;b=&quot;two&quot;\">A&amp;B</a>",
            ),
            (
                "&lt; &gt; &amp; &quot; &#39; &#x1F600;",
                "&lt; &gt; &amp; &quot; &#39; &#x1F600;",
            ),
            (
                "&nbsp; &bogus; &#0; &#xD800; &unfinished",
                "&amp;nbsp; &amp;bogus; &amp;#0; &amp;#xD800; &amp;unfinished",
            ),
            (
                "<script>x & y</script>",
                "&lt;script&gt;x &amp; y&lt;/script&gt;",
            ),
            ("<b>unclosed & text", "&lt;b&gt;unclosed &amp; text"),
            ("<b>x<i>y</b>", "<b>x&lt;i&gt;y</b>"),
            (
                "<pre><code class='language-rust'>a < b && x > y</code></pre>",
                "<pre><code class=\"language-rust\">a &lt; b &amp;&amp; x &gt; y</code></pre>",
            ),
            (
                "<code><b>literal</b> & x</code>",
                "<code>&lt;b&gt;literal&lt;/b&gt; &amp; x</code>",
            ),
            ("<b>a<code>x</code>b</b>", "<b>a</b><code>x</code><b>b</b>"),
            (
                "<blockquote>a<blockquote>b</blockquote>c</blockquote>",
                "<blockquote>abc</blockquote>",
            ),
            (
                "<span class='tg-spoiler'>secret & text</span>",
                "<span class=\"tg-spoiler\">secret &amp; text</span>",
            ),
            (
                "<blockquote expandable>quote!</blockquote>",
                "<blockquote expandable>quote!</blockquote>",
            ),
            (
                "<tg-emoji emoji-id='123'>😀</tg-emoji>",
                "<tg-emoji emoji-id=\"123\">😀</tg-emoji>",
            ),
        ];
        for (input, expected) in cases {
            let actual = prepare(input);
            assert_eq!(actual, expected, "input: {input:?}");
            assert_eq!(prepare(&actual), actual, "repeated conversion: {input:?}");
        }
    }
}
