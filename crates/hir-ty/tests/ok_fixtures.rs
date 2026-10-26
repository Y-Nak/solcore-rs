use std::{collections::BTreeMap, path::PathBuf};

use dir_test::{Fixture, dir_test};
use hir::diag::Diagnostic;
use nameres::{ModuleKey, module_id_from_key};
use solcore_test_utils::{
    define_frontend_test_db, load_fixture_case, load_reachable_modules, lower_any_diagnostics,
    render_diagnostics, repo_root_from_manifest, run_in_large_stack,
};

define_frontend_test_db!(TestDb, solcore_hir_ty);

#[path = "support/frontend.rs"]
mod frontend;

#[dir_test(dir: "$CARGO_MANIFEST_DIR/../../std", glob: "*.sol")]
fn standard_library_sources_pass_normal_frontend_checks(fixture: Fixture<&str>) {
    let path = PathBuf::from(fixture.path())
        .canonicalize()
        .expect("standard source path");
    run_in_large_stack(move || {
        let std_root = frontend::repo_root()
            .join("std")
            .canonicalize()
            .expect("std root");
        // Each source must be a user entry: imported Std modules are trusted
        // by type checking, which would hide invalid instance declarations.
        let outcome =
            frontend::run_frontend_with_roots(&path, &std_root, &std_root, BTreeMap::new());
        assert!(
            !outcome.frontend_has_errors && !outcome.typeck_has_errors,
            "{} failed normal frontend checks\n  frontend errors: {:#?}\n  typeck errors: {:#?}",
            path.display(),
            outcome.frontend_error_diagnostics,
            outcome.typeck_error_diagnostics,
        );
        if path.file_name().is_some_and(|name| name == "std.sol") {
            assert!(
                outcome.frontend_diagnostics.is_empty() && outcome.typeck_diagnostics.is_empty(),
                "std.sol emitted diagnostics\n  frontend: {:#?}\n  typeck: {:#?}",
                outcome.frontend_diagnostics,
                outcome.typeck_diagnostics,
            );
        }
    });
}

#[dir_test(
    dir: "$CARGO_MANIFEST_DIR/tests/fixtures/ok",
    glob: "**/main.sol"
)]
fn hir_ty_ok_fixture_has_no_diagnostics(fixture: Fixture<&str>) {
    let case_dir = PathBuf::from(fixture.path())
        .parent()
        .expect("case dir")
        .to_path_buf();
    run_in_large_stack(move || {
        let repo_root = repo_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
        let mut db = TestDb::default();
        let entry = load_fixture_case(&mut db, &case_dir, &repo_root, BTreeMap::new());
        load_reachable_modules(&mut db, entry.clone());
        let diagnostics = full_frontend_diagnostics(&db, entry);
        assert!(
            diagnostics.is_empty(),
            "expected no diagnostics for OK fixture `{}`\n{}",
            case_dir.display(),
            render_diagnostics(&db, &diagnostics)
        );
    });
}

fn full_frontend_diagnostics(db: &TestDb, entry: ModuleKey) -> Vec<Diagnostic> {
    let entry = module_id_from_key(db, &entry);
    let mut diagnostics = nameres::reachable_diagnostics(db, entry).to_vec();
    diagnostics.extend(
        solcore_hir_ty::infer::reachable_typeck_diagnostics(db, entry)
            .iter()
            .cloned(),
    );
    lower_any_diagnostics(db, diagnostics)
}
