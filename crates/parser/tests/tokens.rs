use dir_test::{Fixture, dir_test};
use std::path::Path;

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/tokens", glob: "**/main.sol.txt")]
fn token_stream(fixture: Fixture<&str>) {
    let tokens = solcore_parser::lexer::tokens(fixture.content())
        .map(|(token, _)| format!("{:?}", token.expect("valid token")))
        .collect::<Vec<_>>()
        .join("\n");
    let folder = Path::new(fixture.path()).parent().unwrap();
    let mut settings = insta::Settings::new();
    settings.set_snapshot_path(folder);
    settings.set_input_file(fixture.path());
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| insta::assert_snapshot!("tokens", tokens));
}
