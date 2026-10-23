use super::*;
use dir_test::{Fixture, dir_test};
use serde::Deserialize;
use solcore_test_utils::{run_in_large_stack, text_checks::TextChecks};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    source: Option<String>,
    #[serde(default)]
    loader: Loader,
    #[serde(default)]
    checks: TextChecks,
    #[serde(default)]
    functions: BTreeMap<String, TextChecks>,
    undefined_variable: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Loader {
    Memory,
    #[default]
    Fixture,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/regressions", glob: "**/expect.json")]
fn source_emission(fixture: Fixture<&str>) {
    let case: Case = serde_json::from_str(fixture.content()).expect("emission expectations");
    let path = case.source.as_ref().map_or_else(
        || Path::new(fixture.path()).with_file_name("main.sol"),
        |path| repo_root().join(path),
    );
    run_in_large_stack(move || {
        let (db, output) = match case.loader {
            Loader::Memory => {
                let source = fs::read_to_string(&path).expect("emission source");
                specialize_src("source_fixture", &source)
            }
            Loader::Fixture => specialize_fixture(&path),
        };
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let emitted = emit_module(db, &output.module, EmitOptions::default());
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
        let diagnostics = check_program_with_db(db, &emitted.program);
        if let Some(expected) = case.undefined_variable {
            assert!(diagnostics.iter().any(|diagnostic| matches!(
                &diagnostic.kind, CheckDiagnosticKind::UndefinedVariable { name } if name == &expected
            )), "{diagnostics:?}");
        } else {
            assert!(diagnostics.is_empty(), "{diagnostics:?}");
        }
        let hull = pretty_program(db, &emitted.program);
        case.checks.assert_matches(&hull);
        for (function, checks) in case.functions {
            checks.assert_matches(hull_function(&hull, &function));
        }
    });
}
