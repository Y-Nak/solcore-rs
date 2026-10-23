use super::*;
use dir_test::{Fixture, dir_test};
use solcore_test_utils::{run_in_large_stack, text_checks::TextChecks};

#[derive(Default)]
struct Case {
    loader: Loader,
    checks: TextChecks,
    functions: BTreeMap<String, TextChecks>,
    undefined_variable: Option<String>,
}

#[derive(Default)]
enum Loader {
    Memory,
    #[default]
    Fixture,
}

mod cases;

macro_rules! source_cases {
    ($name:ident, $directory:literal, $glob:literal) => {
        #[dir_test(dir: $directory, glob: $glob)]
        fn $name(fixture: Fixture<&str>) {
            run_source_fixture(fixture);
        }
    };
}

source_cases!(
    local_sources,
    "$CARGO_MANIFEST_DIR/tests/fixtures/regressions",
    "**/main.sol"
);
source_cases!(
    assembly_tuple_pattern_write,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/asm-match-tuple-write-read.sol"
);
source_cases!(
    control_flow_capture_identity,
    "$CARGO_MANIFEST_DIR/../..",
    "tests/e2e/control-flow-capture-identity/main.sol"
);
source_cases!(
    control_flow_local_lifetimes,
    "$CARGO_MANIFEST_DIR/../..",
    "tests/e2e/control-flow-local-lifetimes/main.sol"
);
source_cases!(
    empty_deployment,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/parser/tests/fixtures/corpus/ok/test/examples/dispatch/empty_no_constructor.sol"
);
source_cases!(
    inferred_lambda_capture,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/closure-free-var.sol"
);
source_cases!(
    inferred_lambda_helper,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/hull/tests/fixtures/local-lambda-helper.sol"
);
source_cases!(
    inferred_lambda_increment,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/inc-closure.sol"
);
source_cases!(
    inferred_lambda_local_capture,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/closure-free-var-local.sol"
);
source_cases!(
    inferred_lambda_simple,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/SimpleLambda.sol"
);
source_cases!(
    inferred_lambda_std_capture,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/closure-free-var-std.sol"
);
source_cases!(
    nonpayable_deployment,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/parser/tests/fixtures/corpus/ok/test/examples/dispatch/nonpayable_ctor.sol"
);
source_cases!(
    payable_deployment,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/parser/tests/fixtures/corpus/ok/test/examples/dispatch/payable_ctor.sol"
);

fn run_source_fixture(fixture: Fixture<&str>) {
    let path = std::path::PathBuf::from(fixture.path())
        .canonicalize()
        .expect("source fixture path");
    let key = solcore_test_utils::fixture_path_key(env!("CARGO_MANIFEST_DIR"), &path);
    let case = cases::expectations(&key);
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
