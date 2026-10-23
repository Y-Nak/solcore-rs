use dir_test::{Fixture, dir_test};
use solcore_hir::string_literal::{decode_string_literal, encode_string_literal};
use std::path::Path;

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/string_literals", glob: "*.txt")]
fn source_string_literal(fixture: Fixture<&str>) {
    let name = Path::new(fixture.path())
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let expected = match name {
        "plain" => Some("plain"),
        "empty" => Some(""),
        "unicode_nul" => Some("\0é雪"),
        "crlf" => Some("line one\r\nline two"),
        "carriage_return" => Some("\r"),
        "escapes" => Some("a\n\t\"\\b"),
        "trailing_slash"
        | "unescaped_quote"
        | "unquoted"
        | "unsupported_cr_escape"
        | "unsupported_escape" => None,
        other => panic!("missing string literal expectation: {other}"),
    };
    assert_eq!(
        decode_string_literal(fixture.content()).as_deref(),
        expected
    );
    if let Some(value) = expected {
        let encoded = encode_string_literal(value);
        assert_eq!(encoded, *fixture.content());
        assert_eq!(decode_string_literal(&encoded).as_deref(), Some(value));
    }
}
