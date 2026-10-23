use super::*;
use dir_test::{Fixture, dir_test};
use std::path::Path;

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/lowering_audit", glob: "**/main.*")]
fn parsed_source(fixture: Fixture<&str>) {
    let source = fixture.content();
    let db = TestDb::default();
    let (file, module) = parse_module(&db, "lowering_fixture", source);
    let actual = diagnostics(&db, file);
    let mut expected_messages = Vec::new();
    let directives = source
        .lines()
        .filter_map(|line| line.strip_prefix("//~ "))
        .collect::<Vec<_>>();
    assert!(!directives.is_empty(), "parser fixture needs expectations");
    for directive in directives {
        let (key, value) = directive.split_once(": ").expect("expectation value");
        match key {
            "diagnostics" => {
                assert_eq!(actual.len(), value.parse::<usize>().unwrap(), "{actual:?}")
            }
            "diagnostic" => expected_messages.push(value),
            "has-errors" => assert_eq!(
                !actual.is_empty(),
                value.parse::<bool>().unwrap(),
                "{actual:?}"
            ),
            "pragmas" => {
                let actual = module
                    .items(&db)
                    .iter()
                    .filter_map(|item| match item {
                        Item::Pragma(pragma) => Some(pragma.name(&db).atom().text(&db)),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                assert_eq!(actual, value.split_whitespace().collect::<Vec<_>>());
            }
            "function" => {
                let mut words = value.split_whitespace();
                let name = words.next().expect("function name");
                let body = top_function(&db, module, name).body(&db);
                match words.next().expect("body expectation") {
                    "body" => assert!(body.is_some(), "{name}"),
                    "no-body" => assert!(body.is_none(), "{name}"),
                    other => panic!("unknown function expectation: {other}"),
                }
                if let Some(return_keyword) = words.next() {
                    assert_eq!(return_keyword, "return");
                    let number = words.next().expect("return literal");
                    let body = body.unwrap();
                    let last = *body
                        .top_level_stmts(&db)
                        .last()
                        .expect("trailing statement");
                    let StmtKind::Return(Some(value)) = body.stmts(&db).get(last).kind else {
                        panic!("{name}: {:?}", body.stmts(&db));
                    };
                    assert!(matches!(&body.exprs(&db).get(value).kind,
                        ExprKind::Lit(hir::ast::function::LitKind::Number(actual)) if actual == number));
                }
                assert!(words.next().is_none(), "unexpected function expectation");
            }
            "identifier" => {
                let (name, expected) = value.split_once(' ').expect("identifier expectation");
                assert_eq!(
                    solcore_parser::is_valid_identifier(name),
                    expected.parse::<bool>().unwrap(),
                    "{name}"
                );
            }
            other => panic!("unknown parser expectation: {other}"),
        }
    }
    if !expected_messages.is_empty() {
        assert_eq!(
            actual
                .iter()
                .map(|d| d.message.as_str())
                .collect::<Vec<_>>(),
            expected_messages
        );
    }
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/type_identifier_context", glob: "**/main.*")]
fn named_type_identifier_context(fixture: Fixture<&str>) {
    let context = Path::new(fixture.path()).parent().unwrap();
    let name = context
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let (qualification, arguments) = context
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split_once('_')
        .unwrap();
    let qualified = match qualification {
        "bare" => false,
        "qualified" => true,
        other => panic!("unknown type reference context: {other}"),
    };
    let type_arguments = arguments.parse::<usize>().unwrap();
    // `mapping` reserves every unqualified form, while `comptime<T>` is
    // contextual only when it has exactly one unqualified type argument.
    let expected = qualified
        || match name {
            "mapping" => false,
            "comptime" => type_arguments != 1,
            "Value" | "derive" | "while" => true,
            other => panic!("missing type identifier expectation: {other}"),
        };
    let db = TestDb::default();
    let source = fixture.content();
    let (file, module) = parse_module(&db, "type_name_context", source);
    let preserves_named_type = diagnostics(&db, file).is_empty()
        && module.items(&db).iter().any(|item| {
            matches!(item, Item::TypeAlias(alias)
                if matches!(alias.ty(&db).kind(&db), TypeRefKind::Named { name: actual, .. }
                    if actual.atom().text(&db) == name))
        });
    assert_eq!(preserves_named_type, expected, "{source}");
    assert_eq!(
        solcore_parser::is_valid_identifier_in(
            name,
            solcore_parser::IdentifierContext::TypeReference {
                qualified,
                type_arguments
            }
        ),
        preserves_named_type,
        "{source}",
    );
    assert_eq!(
        solcore_parser::requires_type_identifier_context(name),
        matches!(name, "mapping" | "comptime")
    );
}
