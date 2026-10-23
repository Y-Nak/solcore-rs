use std::{collections::BTreeMap, path::PathBuf};

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
    queries: BTreeMap<String, Count>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Count {
    min: usize,
    max: Option<usize>,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/query_execution", glob: "*.json")]
fn frontend_query_execution(fixture: Fixture<&str>) {
    let case: Case = serde_json::from_str(fixture.content()).expect("query expectations");
    run_in_large_stack(move || {
        let repo = frontend::repo_root();
        let outcome = frontend::run_frontend(&repo.join(&case.source), &repo.join(case.std_root));
        assert!(
            outcome.frontend_diagnostics.is_empty() && outcome.typeck_diagnostics.is_empty(),
            "{} failed frontend checks\n  frontend diagnostics: {:#?}\n  typeck diagnostics: {:#?}",
            case.source.display(),
            outcome.frontend_diagnostics,
            outcome.typeck_diagnostics,
        );
        assert!(
            !case.queries.is_empty(),
            "query fixture must check an execution count"
        );
        for (query, count) in case.queries {
            let actual = frontend::query_executions(&outcome.executed, &query);
            assert!(
                actual >= count.min && count.max.is_none_or(|max| actual <= max),
                "{}: {query} executed {actual} times; expected {}..={:?}\n{:#?}",
                case.source.display(),
                count.min,
                count.max,
                outcome.executed,
            );
        }
    });
}
