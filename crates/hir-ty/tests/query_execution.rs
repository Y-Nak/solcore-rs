use std::path::PathBuf;

use dir_test::{Fixture, dir_test};
use solcore_test_utils::run_in_large_stack;

#[path = "support/frontend.rs"]
mod frontend;

#[dir_test(dir: "$CARGO_MANIFEST_DIR/../parser/tests/fixtures/corpus/ok/test/examples/cases", glob: "tabled-*.sol")]
fn solver_and_soundness_queries_execute(fixture: Fixture<&str>) {
    let path = PathBuf::from(fixture.path())
        .canonicalize()
        .expect("solver source");
    run_in_large_stack(move || {
        let std_root = frontend::repo_root().join("std");
        let outcome = frontend::run_frontend(&path, &std_root);
        assert_no_diagnostics(&outcome);
        for query in ["solve_report", "instance_soundness_diagnostics"] {
            assert!(
                frontend::query_executions(&outcome.executed, query) > 0,
                "{} did not execute {query}: {:#?}",
                path.display(),
                outcome.executed,
            );
        }
    });
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/../parser/tests/fixtures/corpus/ok/test/examples/dispatch", glob: "empty_no_constructor.sol")]
fn generated_dispatch_reuses_std_instance_facts_per_module(fixture: Fixture<&str>) {
    let path = PathBuf::from(fixture.path())
        .canonicalize()
        .expect("dispatch source");
    run_in_large_stack(move || {
        let outcome = frontend::run_frontend(&path, &frontend::repo_root().join("std"));
        assert_no_diagnostics(&outcome);
        let module_facts = frontend::query_executions(&outcome.executed, "module_instance_facts");
        let per_origin =
            frontend::query_executions(&outcome.executed, "instance_origin_clause_set");
        // Std has over one hundred instance origins. Lower them by their
        // handful of owner modules instead of one tracked query per origin.
        assert!(
            (1..32).contains(&module_facts),
            "expected module-scoped instance facts, got {module_facts}: {:#?}",
            outcome.executed,
        );
        assert_eq!(
            per_origin, 0,
            "expected no per-origin instance queries: {:#?}",
            outcome.executed,
        );
    });
}

fn assert_no_diagnostics(outcome: &frontend::RunOutcome) {
    assert!(
        outcome.frontend_diagnostics.is_empty() && outcome.typeck_diagnostics.is_empty(),
        "frontend checks failed\n  frontend diagnostics: {:#?}\n  typeck diagnostics: {:#?}",
        outcome.frontend_diagnostics,
        outcome.typeck_diagnostics,
    );
}
