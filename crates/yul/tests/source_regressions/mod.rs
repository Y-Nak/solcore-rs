use super::*;
use dir_test::{Fixture, dir_test};

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/regressions", glob: "**/main.sol")]
fn source_rendering(fixture: Fixture<&str>) {
    let path = std::path::Path::new(fixture.path());
    let yul = render_source("source_fixture", fixture.content());
    insta::with_settings!({
        snapshot_path => path.parent().expect("fixture directory"),
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!("expected", yul);
    });
}
