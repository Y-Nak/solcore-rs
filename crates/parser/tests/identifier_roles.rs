use dir_test::{Fixture, dir_test};
use serde::Deserialize;
use solcore_parser::{IdentifierContext, is_valid_identifier_in};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    ordinary: bool,
    runtime_parameter: bool,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/identifier_roles", glob: "*.json")]
fn identifier_role(fixture: Fixture<&str>) {
    let case: Case = serde_json::from_str(fixture.content()).expect("identifier expectations");
    assert_eq!(
        is_valid_identifier_in(&case.name, IdentifierContext::Ordinary),
        case.ordinary
    );
    assert_eq!(
        is_valid_identifier_in(&case.name, IdentifierContext::RuntimeParameter),
        case.runtime_parameter
    );
}
