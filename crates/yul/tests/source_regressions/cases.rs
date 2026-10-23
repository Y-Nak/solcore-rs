// Expected behavior is expressed in Rust so fixture discovery only depends on source files.
use super::*;

pub(super) fn expectations(path: &str) -> Case {
    match path {
        "crates/yul/tests/fixtures/regressions/assembly_escapes/main.sol" => Case {
            checks: TextChecks {
                contains: vec!["\"a\\n\\t\\\"\\\\b\"".into()],
                not_contains: vec!["\"a\\\\n".into()],
                ..TextChecks::default()
            },
        },
        "crates/yul/tests/fixtures/regressions/folded_cr/main.sol" => Case {
            checks: TextChecks {
                contains: vec![":= 4".into()],
                ..TextChecks::default()
            },
        },
        "crates/yul/tests/fixtures/regressions/folded_crlf/main.sol" => Case {
            checks: TextChecks {
                contains: vec![":= 19".into()],
                ..TextChecks::default()
            },
        },
        _ => panic!("missing expectations for source fixture {path}"),
    }
}
