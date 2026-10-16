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

#[cfg(test)]
mod tests {
    use super::{decode_string_literal, encode_string_literal};

    #[test]
    fn encoded_strings_round_trip_through_source_decoding() {
        for value in [
            "",
            "plain",
            "\r",
            "line one\r\nline two",
            "a\n\t\"\\b",
            "\0é雪",
        ] {
            let encoded = encode_string_literal(value);
            assert_eq!(decode_string_literal(&encoded).as_deref(), Some(value));
        }
        assert_eq!(encode_string_literal("\r"), "\"\r\"");
    }

    #[test]
    fn decodes_known_escapes() {
        assert_eq!(
            decode_string_literal(r#""a\n\t\"\\b""#),
            Some("a\n\t\"\\b".to_owned())
        );
    }

    #[test]
    fn rejects_unsupported_source_escapes() {
        assert_eq!(decode_string_literal(r#""before\qafter""#), None);
        assert_eq!(decode_string_literal(r#""\r""#), None);
    }

    #[test]
    fn rejects_malformed_literal_spellings() {
        assert_eq!(decode_string_literal("not quoted"), None);
        assert_eq!(decode_string_literal("\"trailing\\\""), None);
        assert_eq!(decode_string_literal("\"unescaped\"quote\""), None);
    }
}
