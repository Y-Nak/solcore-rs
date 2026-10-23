use solcore_hir::string_literal::{decode_string_literal, encode_string_literal};

fn assert_string_literal(source: &str, expected: Option<&str>) {
    assert_eq!(decode_string_literal(source).as_deref(), expected);
    if let Some(value) = expected {
        let encoded = encode_string_literal(value);
        assert_eq!(encoded, source);
        assert_eq!(decode_string_literal(&encoded).as_deref(), Some(value));
    }
}

#[test]
fn carriage_return() {
    assert_string_literal(
        include_str!("fixtures/string_literals/carriage_return.txt"),
        Some("\r"),
    );
}

#[test]
fn crlf() {
    assert_string_literal(
        include_str!("fixtures/string_literals/crlf.txt"),
        Some("line one\r\nline two"),
    );
}

#[test]
fn empty() {
    assert_string_literal(include_str!("fixtures/string_literals/empty.txt"), Some(""));
}

#[test]
fn escapes() {
    assert_string_literal(
        include_str!("fixtures/string_literals/escapes.txt"),
        Some("a\n\t\"\\b"),
    );
}

#[test]
fn plain() {
    assert_string_literal(
        include_str!("fixtures/string_literals/plain.txt"),
        Some("plain"),
    );
}

#[test]
fn trailing_slash() {
    assert_string_literal(
        include_str!("fixtures/string_literals/trailing_slash.txt"),
        None,
    );
}

#[test]
fn unescaped_quote() {
    assert_string_literal(
        include_str!("fixtures/string_literals/unescaped_quote.txt"),
        None,
    );
}

#[test]
fn unicode_nul() {
    assert_string_literal(
        include_str!("fixtures/string_literals/unicode_nul.txt"),
        Some("\0é雪"),
    );
}

#[test]
fn unquoted() {
    assert_string_literal(include_str!("fixtures/string_literals/unquoted.txt"), None);
}

#[test]
fn unsupported_cr_escape() {
    assert_string_literal(
        include_str!("fixtures/string_literals/unsupported_cr_escape.txt"),
        None,
    );
}

#[test]
fn unsupported_escape() {
    assert_string_literal(
        include_str!("fixtures/string_literals/unsupported_escape.txt"),
        None,
    );
}
