use nameres::{Db as _, module_id_from_key};
use parser::parse_file_to_hir;
use solcore_hull::{EmitDiagnosticKind, EmitOptions, check_program_with_db, emit_module};
use solcore_test_utils::{define_frontend_test_db, load_main_source, run_in_large_stack};
use specialize::{SpecializeOptions, specialize_module};

define_frontend_test_db!(TestDb, hir_ty);

enum LayoutExpectation {
    Finite { nonempty: bool },
    NonRegular,
    UnsupportedRecursive,
}

#[dir_test::dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/layouts", glob: "**/main.sol")]
fn layout_finiteness(fixture: dir_test::Fixture<&str>) {
    let mut expectations = solcore_test_utils::source_directives(fixture.content(), "layout");
    let (line, expectation) = expectations
        .next()
        .expect("layout fixture needs an expectation");
    assert!(
        expectations.next().is_none(),
        "duplicate layout expectation"
    );
    let expectation = match expectation {
        "finite" => LayoutExpectation::Finite { nonempty: false },
        "finite-nonempty" => LayoutExpectation::Finite { nonempty: true },
        "non-regular" => LayoutExpectation::NonRegular,
        "unsupported-recursive" => LayoutExpectation::UnsupportedRecursive,
        other => panic!("line {line}: unknown layout expectation: {other}"),
    };
    assert_layout_finiteness(fixture.content(), expectation);
}

fn assert_layout_finiteness(source: &str, expectation: LayoutExpectation) {
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
        if let LayoutExpectation::Finite { nonempty } = expectation {
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
                emitted.diagnostics.iter().any(|d| match expectation {
                    LayoutExpectation::NonRegular => {
                        matches!(d.kind, EmitDiagnosticKind::NonRegularRecursiveType { .. })
                    }
                    LayoutExpectation::UnsupportedRecursive => {
                        matches!(d.kind, EmitDiagnosticKind::UnsupportedRecursiveType { .. })
                    }
                    LayoutExpectation::Finite { .. } => unreachable!(),
                }),
                "{source}\n{:?}",
                emitted.diagnostics
            );
        }
    });
}
