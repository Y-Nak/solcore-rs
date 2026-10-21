use dir_test::{Fixture, dir_test};
use serde::Deserialize;
use solcore_hir::string_literal::{decode_string_literal, encode_string_literal};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    value: Option<String>,
    encoded: Option<String>,
    decode: Option<String>,
    decoded: Option<String>,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/string_literals", glob: "**/case.json")]
fn source_string_literal(fixture: Fixture<&str>) {
    let case: Case = serde_json::from_str(fixture.content()).expect("string literal fixture");
    assert!(
        case.encoded.is_none() || case.value.is_some(),
        "encoded spelling requires an input value"
    );
    assert!(
        case.decoded.is_none() || case.decode.is_some(),
        "decoded value requires an input spelling"
    );
    assert!(
        case.value.is_some() || case.decode.is_some(),
        "empty string literal fixture"
    );
    if let Some(value) = case.value {
        let encoded = encode_string_literal(&value);
        assert_eq!(
            decode_string_literal(&encoded).as_deref(),
            Some(value.as_str())
        );
        if let Some(expected) = case.encoded {
            assert_eq!(encoded, expected);
        }
    }
    if let Some(source) = case.decode {
        assert_eq!(decode_string_literal(&source), case.decoded);
    }
}
