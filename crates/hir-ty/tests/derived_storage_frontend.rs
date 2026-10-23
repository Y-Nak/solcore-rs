use std::{collections::BTreeMap, path::PathBuf};

use hir::diag::Diagnostic;
use nameres::{ModuleKey, module_id_from_key};
use solcore_test_utils::{
    define_frontend_test_db, load_fixture_case, load_reachable_modules, lower_any_diagnostics,
    render_diagnostics, repo_root_from_manifest, run_in_large_stack,
};

define_frontend_test_db!(TestDb, solcore_hir_ty);

fn assert_storage_frontend_case(
    fixture: dir_test::Fixture<&str>,
    should_pass: bool,
    require_clean: bool,
) {
    let fixture = PathBuf::from(fixture.path()).parent().unwrap().to_owned();
    run_in_large_stack(move || {
        let case = fixture.display();
        let repo_root = repo_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
        let mut db = TestDb::default();
        let entry = load_fixture_case(&mut db, &fixture, &repo_root, BTreeMap::new());
        load_reachable_modules(&mut db, entry.clone());
        let diagnostics = full_frontend_diagnostics(&db, entry);
        let has_storage_failure = diagnostics.iter().any(|diagnostic| {
            diagnostic.message.contains("CanStore") || diagnostic.message.contains("StorageSize")
        });
        assert_eq!(
            !has_storage_failure,
            should_pass,
            "unexpected storage frontend result for {case}\n{}",
            render_diagnostics(&db, &diagnostics),
        );
        if require_clean {
            assert!(
                diagnostics.is_empty(),
                "positive storage field validation emitted diagnostics for {case}\n{}",
                render_diagnostics(&db, &diagnostics),
            );
        }
    });
}

#[dir_test::dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/derived_storage_frontend/rejected", glob: "*/main.sol")]
fn storage_frontend_rejected(fixture: dir_test::Fixture<&str>) {
    assert_storage_frontend_case(fixture, false, false);
}

#[dir_test::dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/derived_storage_frontend/accepted", glob: "*/main.sol")]
fn storage_frontend_accepted(fixture: dir_test::Fixture<&str>) {
    assert_storage_frontend_case(fixture, true, false);
}

#[dir_test::dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/derived_storage_frontend/clean", glob: "*/main.sol")]
fn storage_frontend_clean(fixture: dir_test::Fixture<&str>) {
    assert_storage_frontend_case(fixture, true, true);
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
