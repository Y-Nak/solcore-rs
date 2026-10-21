/// Decodes the source spelling of a Solcore string literal.
///
/// Accepts the same escapes as the source lexer. Invalid spellings return
/// `None`, including when HIR is constructed directly by compiler consumers.
pub fn decode_string_literal(text: &str) -> Option<String> {
    let inner = text.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '"' {
            return None;
        }
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next()? {
            '"' => out.push('"'),
            '\\' => out.push('\\'),
            'n' => out.push('\n'),
            't' => out.push('\t'),
            _ => return None,
        }
    }
    Some(out)
}

/// Encodes a string value using only escapes accepted by the source lexer.
///
/// Carriage returns and other characters without a source escape remain
/// literal characters. The lexer accepts them inside quoted strings, so the
/// result always round-trips through [`decode_string_literal`].
pub fn encode_string_literal(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}
