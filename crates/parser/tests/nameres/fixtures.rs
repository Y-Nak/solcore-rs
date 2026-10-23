use super::*;
use dir_test::{Fixture, dir_test};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    #[serde(default)]
    error_expression: bool,
    error_statement_in: Option<String>,
    #[serde(default)]
    unresolved_identifier: bool,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/recovery_nameres", glob: "**/expected.json")]
fn parse_recovery_suppresses_lookup_cascades(fixture: Fixture<&str>) {
    let source =
        std::fs::read_to_string(Path::new(fixture.path()).with_file_name("source.txt")).unwrap();
    let case: Case = serde_json::from_str(fixture.content()).expect("recovery expectations");
    let db = TestDb::default();
    let (file, module) = parse_and_module(&db, "recovery_fixture", &source);
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
    if case.error_expression {
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
    if let Some(name) = case.error_statement_in {
        let body = top_function(&db, module, &name)
            .body(&db)
            .expect("function body");
        assert!(
            body.stmts(&db)
                .iter()
                .any(|(_, stmt)| matches!(stmt.kind, hir::ast::function::StmtKind::Error))
        );
    }
    if case.unresolved_identifier {
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
}
