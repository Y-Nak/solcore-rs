//! Regression tests for divergent signature-inference fixpoints.

use solcore_hir_ty as hir_ty;
use solcore_test_utils::{define_frontend_test_db, load_main_source, run_in_large_stack};

define_frontend_test_db!(TestDb, hir_ty);

/// The missing `returns` clause intentionally gives `f` the canonical unit
/// result while its body returns `f` itself. Recovery from that recursive type
/// mismatch must not make the scheme query cycle or panic.
#[dir_test::dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/scheme_cycle", glob: "*.sol")]
fn recursive_unit_return_mismatch_does_not_panic(fixture: dir_test::Fixture<&str>) {
    let source = fixture.content().to_string();
    run_in_large_stack(move || {
        let mut db = TestDb::default();
        let entry = load_main_source(&mut db, &source);
        let entry = nameres::module_id_from_key(&db, &entry);
        let _ = nameres::reachable_diagnostics(&db, entry);
        let _ = hir_ty::infer::reachable_typeck_diagnostics(&db, entry);
    });
}
