use std::path::PathBuf;

use dir_test::{Fixture, dir_test};
use solcore_test_utils::run_in_large_stack;

#[path = "support/frontend.rs"]
mod frontend;

#[dir_test(dir: "$CARGO_MANIFEST_DIR/../parser/tests/fixtures/corpus/ok/test/examples/cases", glob: "false-redundant-warning.sol")]
#[dir_test(dir: "$CARGO_MANIFEST_DIR/../parser/tests/fixtures/corpus/ok/test/examples/cases", glob: "polymatch-error.sol")]
#[dir_test(dir: "$CARGO_MANIFEST_DIR/../parser/tests/fixtures/corpus/ok/test/examples/comptime", glob: "match_labels.sol")]
fn conservative_match_coverage(fixture: Fixture<&str>) {
    let path = PathBuf::from(fixture.path())
        .canonicalize()
        .expect("match source");
    run_in_large_stack(move || {
        let std_root = frontend::repo_root().join("crates/parser/tests/fixtures/corpus/ok/std");
        let outcome = frontend::run_frontend(&path, &std_root);
        assert!(
            !outcome.frontend_has_errors && !outcome.typeck_has_errors,
            "{} failed before conservative coverage could be checked\n  frontend errors: {:#?}\n  typeck errors: {:#?}",
            path.display(),
            outcome.frontend_error_diagnostics,
            outcome.typeck_error_diagnostics,
        );
        let coverage = outcome
            .typeck_diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.starts_with("SC0302:") || diagnostic.starts_with("SC0303:")
            })
            .collect::<Vec<_>>();
        assert!(
            coverage.is_empty(),
            "{} emitted false coverage diagnostics: {coverage:#?}",
            path.display()
        );
    });
}
