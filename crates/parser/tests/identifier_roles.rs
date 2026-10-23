use dir_test::{Fixture, dir_test};
use solcore_parser::{IdentifierContext, is_valid_identifier_in};
use std::path::Path;

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/identifier_roles", glob: "*.txt")]
fn identifier_role(fixture: Fixture<&str>) {
    let case = Path::new(fixture.path())
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let (name, ordinary, runtime_parameter) = match case {
        "comptime" => ("comptime", true, false),
        "derive" => ("derive", true, true),
        "mapping" => ("mapping", true, true),
        "return" => ("return", false, false),
        "value-name" => ("value-name", false, false),
        "value" => ("value", true, true),
        "where" => ("where", true, true),
        "while" => ("while", true, true),
        other => panic!("missing identifier expectation: {other}"),
    };
    assert_eq!(*fixture.content(), name);
    assert_eq!(
        is_valid_identifier_in(fixture.content(), IdentifierContext::Ordinary),
        ordinary
    );
    assert_eq!(
        is_valid_identifier_in(fixture.content(), IdentifierContext::RuntimeParameter),
        runtime_parameter
    );
}
