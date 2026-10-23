use std::path::PathBuf;

use dir_test::{Fixture, dir_test};
use serde::Deserialize;
use solcore_test_utils::run_in_large_stack;

#[path = "support/frontend.rs"]
mod frontend;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    source: PathBuf,
    std_root: PathBuf,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/match_coverage/conservative", glob: "*.json")]
fn conservative_match_coverage(fixture: Fixture<&str>) {
    let case: Case = serde_json::from_str(fixture.content()).expect("match coverage fixture");
    run_in_large_stack(move || {
        let repo = frontend::repo_root();
        let outcome = frontend::run_frontend(&repo.join(&case.source), &repo.join(case.std_root));
        assert!(
            !outcome.frontend_has_errors && !outcome.typeck_has_errors,
            "{} failed before conservative coverage could be checked\n  frontend errors: {:#?}\n  typeck errors: {:#?}",
            case.source.display(),
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
            case.source.display()
        );
    });
}
