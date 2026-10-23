use super::*;
use dir_test::{Fixture, dir_test};
use serde::Deserialize;
use solcore_test_utils::text_checks::TextChecks;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    checks: TextChecks,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/regressions", glob: "**/expect.json")]
fn source_rendering(fixture: Fixture<&str>) {
    let case: Case = serde_json::from_str(fixture.content()).expect("rendering expectations");
    let path = Path::new(fixture.path()).with_file_name("main.sol");
    let source = fs::read_to_string(path).expect("rendering source");
    let yul = render_source("source_fixture", &source);
    case.checks.assert_matches(&yul);
}
