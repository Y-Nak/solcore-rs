use super::*;
use dir_test::{Fixture, dir_test};
use solcore_test_utils::text_checks::TextChecks;

#[derive(Default)]
struct Case {
    checks: TextChecks,
}

mod cases;

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/regressions", glob: "**/main.sol")]
fn source_rendering(fixture: Fixture<&str>) {
    let path = std::path::PathBuf::from(fixture.path())
        .canonicalize()
        .expect("source fixture path");
    let key = solcore_test_utils::fixture_path_key(env!("CARGO_MANIFEST_DIR"), &path);
    let case = cases::expectations(&key);
    let source = fs::read_to_string(path).expect("rendering source");
    let yul = render_source("source_fixture", &source);
    case.checks.assert_matches(&yul);
}
