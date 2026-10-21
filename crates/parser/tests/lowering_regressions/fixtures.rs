use super::*;
use dir_test::{Fixture, dir_test};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    diagnostics: Option<Vec<String>>,
    has_errors: Option<bool>,
    pragmas: Option<Vec<String>>,
    #[serde(default)]
    functions: Vec<FunctionExpectation>,
    #[serde(default)]
    identifiers: BTreeMap<String, bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionExpectation {
    name: String,
    body: bool,
    last_return_number: Option<String>,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/lowering_audit", glob: "**/expected.json")]
fn parsed_source(fixture: Fixture<&str>) {
    let source =
        std::fs::read_to_string(Path::new(fixture.path()).with_file_name("source.txt")).unwrap();
    let case: Case = serde_json::from_str(fixture.content()).expect("parser expectations");
    let db = TestDb::default();
    let (file, module) = parse_module(&db, "lowering_fixture", &source);
    let actual = diagnostics(&db, file);
    if let Some(expected) = case.diagnostics {
        assert_eq!(
            actual.iter().map(|d| d.message.clone()).collect::<Vec<_>>(),
            expected
        );
    }
    if let Some(has_errors) = case.has_errors {
        assert_eq!(!actual.is_empty(), has_errors, "{actual:?}");
    }
    if let Some(expected) = case.pragmas {
        let actual = module
            .items(&db)
            .iter()
            .filter_map(|item| match item {
                Item::Pragma(pragma) => Some(pragma.name(&db).atom().text(&db).to_owned()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
    for expected in case.functions {
        let body = top_function(&db, module, &expected.name).body(&db);
        assert_eq!(body.is_some(), expected.body, "{}", expected.name);
        if let Some(number) = expected.last_return_number {
            let body = body.unwrap();
            let last = *body
                .top_level_stmts(&db)
                .last()
                .expect("trailing statement");
            let StmtKind::Return(Some(value)) = body.stmts(&db).get(last).kind else {
                panic!("{}: {:?}", expected.name, body.stmts(&db));
            };
            assert!(matches!(&body.exprs(&db).get(value).kind,
                ExprKind::Lit(hir::ast::function::LitKind::Number(actual)) if actual == &number));
        }
    }
    for (name, expected) in case.identifiers {
        assert_eq!(
            solcore_parser::is_valid_identifier(&name),
            expected,
            "{name}"
        );
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TypeNameCase {
    name: String,
    requires_context: bool,
    references: Vec<TypeReferenceCase>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TypeReferenceCase {
    source: String,
    qualified: bool,
    type_arguments: usize,
    named_type: bool,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/type_identifier_context", glob: "**/expected.json")]
fn named_type_identifier_context(fixture: Fixture<&str>) {
    let folder = Path::new(fixture.path()).parent().unwrap();
    let case: TypeNameCase =
        serde_json::from_str(fixture.content()).expect("type reference expectations");
    let mut requires_context = false;
    for reference in case.references {
        let source = std::fs::read_to_string(folder.join(reference.source)).unwrap();
        let db = TestDb::default();
        let (file, module) = parse_module(&db, "type_name_context", &source);
        let preserves_named_type = diagnostics(&db, file).is_empty()
            && module.items(&db).iter().any(|item| {
                matches!(item, Item::TypeAlias(alias)
                    if matches!(alias.ty(&db).kind(&db), TypeRefKind::Named { name, .. }
                        if name.atom().text(&db) == case.name))
            });
        assert_eq!(preserves_named_type, reference.named_type, "{source}");
        assert_eq!(
            solcore_parser::is_valid_identifier_in(
                &case.name,
                solcore_parser::IdentifierContext::TypeReference {
                    qualified: reference.qualified,
                    type_arguments: reference.type_arguments,
                }
            ),
            preserves_named_type,
            "{source}",
        );
        requires_context |= !preserves_named_type;
    }
    assert_eq!(requires_context, case.requires_context);
    assert_eq!(
        solcore_parser::requires_type_identifier_context(&case.name),
        requires_context
    );
}
