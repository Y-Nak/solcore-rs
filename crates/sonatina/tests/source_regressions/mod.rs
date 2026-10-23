use super::*;
use dir_test::{Fixture, dir_test};
use solcore_test_utils::{run_in_large_stack, text_checks::TextChecks};
use sonatina_codegen::{EvmCompile, OptLevel};

#[derive(Default)]
struct Case {
    file_url_imports: bool,
    checks: TextChecks,
    compile: Vec<Optimization>,
    compile_error: Option<String>,
    deploy_output: Option<String>,
}

enum Optimization {
    O0,
    O2,
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
    incremental_reachability,
    "$CARGO_MANIFEST_DIR/../..",
    "tests/e2e/incremental-reachability/main.sol"
);
source_cases!(
    loop_post_reachability,
    "$CARGO_MANIFEST_DIR/../..",
    "tests/e2e/loop-post-reachability/main.sol"
);

fn run_source_fixture(fixture: Fixture<&str>) {
    let path = std::path::PathBuf::from(fixture.path())
        .canonicalize()
        .expect("source fixture path");
    let key = solcore_test_utils::fixture_path_key(env!("CARGO_MANIFEST_DIR"), &path);
    let case = cases::expectations(&key);
    let source = std::fs::read_to_string(&path).expect("lowering source");
    run_in_large_stack(move || {
        let (module, ir) = lower_source_inner(&source, case.file_url_imports);
        case.checks.assert_matches(&ir);
        assert!(
            case.compile_error.is_none() || !case.compile.is_empty(),
            "compile error requires a compile level"
        );
        for level in case.compile {
            let level = match level {
                Optimization::O0 => OptLevel::O0,
                Optimization::O2 => OptLevel::O2,
            };
            let result = EvmCompile::new(module.clone_for_funcs(&module.funcs()))
                .with_opt_level(level)
                .compile();
            if let Some(expected) = &case.compile_error {
                // Preserve upstream memory diagnostics without changing upstream code.
                let errors = result.expect_err("expected backend rejection");
                assert!(
                    format!("{errors:?}").contains(expected),
                    "{level:?}: {errors:?}"
                );
            } else {
                result.expect("compile lowered fixture");
            }
        }
        if let Some(output) = case.deploy_output {
            use solcore_test_utils::e2e::{
                e2e_enabled, e2e_required, encode_hex, parse_e2e_scenario, with_evm_harness,
            };
            assert!(
                e2e_enabled() || !e2e_required(),
                "E2E_REQUIRED=1 requires E2E=1"
            );
            if !e2e_enabled() {
                return;
            }
            let artifacts = EvmCompile::new(module)
                .with_opt_level(OptLevel::O2)
                .compile()
                .expect("compile deployment fixture");
            let creation = artifacts[0]
                .sections
                .iter()
                .find(|(name, _)| name.0.as_str() == "init")
                .expect("entry init section");
            let deployment = format!("#[deploy(0x) -> success({output})]");
            let scenario = parse_e2e_scenario(
                "OutputDeploy",
                [r#"#[scenario("entry_identity")]"#, &deployment],
            )
            .expect("valid entry scenario")
            .expect("entry scenario");
            with_evm_harness(|harness| {
                if let Some(harness) = harness {
                    harness.execute_raw_vector(&encode_hex(&creation.1.bytes), &scenario)?;
                }
                Ok(())
            })
            .expect("deployment output");
        }
    });
}
