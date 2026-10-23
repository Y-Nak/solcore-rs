use dir_test::{Fixture, dir_test};
use std::{fs, path::Path};

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/tokens", glob: "**/expected.json")]
fn token_stream(fixture: Fixture<&str>) {
    let source =
        fs::read_to_string(Path::new(fixture.path()).with_file_name("source.txt")).unwrap();
    let expected: Vec<String> =
        serde_json::from_str(fixture.content()).expect("token expectations");
    let actual = solcore_parser::lexer::tokens(&source)
        .map(|(token, _)| format!("{:?}", token.expect("valid token")))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}
