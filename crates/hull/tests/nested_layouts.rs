use nameres::{Db as _, module_id_from_key};
use parser::parse_file_to_hir;
use solcore_hull::{EmitOptions, check_program_with_db, emit_module};
use solcore_test_utils::{define_frontend_test_db, load_main_source, run_in_large_stack};
use specialize::{SpecializeOptions, specialize_module};

define_frontend_test_db!(TestDb, hir_ty);

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    finite: bool,
    nonempty: bool,
}

#[dir_test::dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/layouts", glob: "**/expect.json")]
fn layout_finiteness(fixture: dir_test::Fixture<&str>) {
    let expected: Expected = serde_json::from_str(fixture.content()).expect("layout expectations");
    let source =
        std::fs::read_to_string(std::path::Path::new(fixture.path()).with_file_name("main.sol"))
            .expect("layout source");
    assert_layout_finiteness(&source, expected.finite, expected.nonempty);
}

fn assert_layout_finiteness(source: &str, finite: bool, nonempty: bool) {
    let source = source.to_owned();
    run_in_large_stack(move || {
        let mut db = TestDb::default();
        let key = load_main_source(&mut db, &source);
        let entry = module_id_from_key(&db, &key);
        let file = db.module_file(entry).expect("main source");
        let module = parse_file_to_hir(&db, file).module(&db);
        assert!(
            parser::parse_diagnostics(&db, file).is_empty(),
            "{source}\n{:?}",
            parser::parse_diagnostics(&db, file)
        );
        let diagnostics = hir_ty::collect_frontend_diagnostics(&db, entry);
        assert!(
            diagnostics
                .iter()
                .all(|d| d.level != hir::diag::DiagnosticLevel::Error),
            "{source}\n{diagnostics:?}"
        );
        let specialized = specialize_module(&db, module, SpecializeOptions::default());
        assert!(
            specialized.diagnostics.is_empty(),
            "{source}\n{:?}",
            specialized.diagnostics
        );
        let emitted = emit_module(&db, &specialized.module, EmitOptions::default());
        if finite {
            assert!(
                emitted.diagnostics.is_empty(),
                "{source}\n{:?}",
                emitted.diagnostics
            );
            assert!(check_program_with_db(&db, &emitted.program).is_empty());
            if nonempty {
                assert!(!emitted.program.functions.is_empty());
            }
        } else {
            assert!(
                emitted.diagnostics.iter().any(|d| matches!(
                    d.kind,
                    solcore_hull::EmitDiagnosticKind::NonRegularRecursiveType { .. }
                )),
                "{source}\n{:?}",
                emitted.diagnostics
            );
        }
    });
}
