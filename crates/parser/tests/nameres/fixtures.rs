use super::*;
use dir_test::{Fixture, dir_test};

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/recovery_nameres", glob: "**/main.*")]
fn parse_recovery_suppresses_lookup_cascades(fixture: Fixture<&str>) {
    let source = fixture.content();
    let db = TestDb::default();
    let (file, module) = parse_and_module(&db, "recovery_fixture", source);
    assert!(
        !parse_diagnostics(&db, file).is_empty(),
        "fixture must have parse errors"
    );
    let scope = item_scope(&db, module);
    let resolution = resolve_module_with_imports_and_policy(
        &db,
        module,
        scope,
        &EmptyImportedNames,
        NameresDiagnosticPolicy::SuppressForParseErrors,
    );
    assert!(
        resolution.diagnostics.is_empty(),
        "{:?}",
        resolution.diagnostics
    );
    for directive in source.lines().filter_map(|line| line.strip_prefix("//~ ")) {
        match directive {
            "error-expression" => {
                assert!(
                    resolution
                        .bodies
                        .iter()
                        .flat_map(|map| &map.exprs)
                        .any(|entry| {
                            matches!(&entry.body.exprs(&db).get(entry.expr).kind, ExprKind::Error)
                                && matches!(entry.resolution, Resolution::Err)
                        }),
                    "recovered expressions must retain their error resolution"
                );
            }
            value if value.starts_with("error-statement-in: ") => {
                let name = value.strip_prefix("error-statement-in: ").unwrap();
                let body = top_function(&db, module, name)
                    .body(&db)
                    .expect("function body");
                assert!(
                    body.stmts(&db)
                        .iter()
                        .any(|(_, stmt)| matches!(stmt.kind, hir::ast::function::StmtKind::Error))
                );
            }
            "unresolved-identifier" => {
                assert!(
                    resolution
                        .bodies
                        .iter()
                        .flat_map(|map| &map.exprs)
                        .any(|entry| {
                            matches!(
                                &entry.body.exprs(&db).get(entry.expr).kind,
                                ExprKind::Ident(_)
                            ) && matches!(entry.resolution, Resolution::Err)
                        })
                );
            }
            other => panic!("unknown recovery expectation: {other}"),
        }
    }
}
