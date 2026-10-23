use super::*;
use dir_test::{Fixture, dir_test};
use solcore_test_utils::{
    run_in_large_stack, source_directives,
    text_checks::{TextChecks, directive_string},
};
use sonatina_codegen::{EvmCompile, OptLevel};

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/regressions", glob: "**/main.sol")]
fn local_sources(fixture: Fixture<&str>) {
    let source = std::fs::read_to_string(fixture.path()).expect("lowering source");
    run_source_fixture(&source);
}

#[test]
fn incremental_reachability_compiles_at_o0_and_o2() {
    run_source_fixture(include_str!(
        "../../../../tests/e2e/incremental-reachability/main.sol"
    ));
}

#[test]
fn loop_post_reachability_compiles_without_optimization() {
    run_source_fixture(include_str!(
        "../../../../tests/e2e/loop-post-reachability/main.sol"
    ));
}

fn run_source_fixture(source: &str) {
    let mut file_url_imports = false;
    let mut compile = Vec::new();
    let mut compile_error = None;
    let mut has_directives = false;
    for (line, directive) in source_directives(source, "sonatina") {
        has_directives = true;
        let (command, argument) = directive.split_once(' ').unwrap_or((directive, ""));
        match command {
            "valid" | "file-url-imports" => {
                assert!(argument.is_empty(), "line {line}: unexpected argument");
                file_url_imports |= command == "file-url-imports";
            }
            "compile" => compile.push(match argument {
                "O0" => OptLevel::O0,
                "O2" => OptLevel::O2,
                _ => panic!("line {line}: expected compile O0 or O2"),
            }),
            "compile-error" => {
                assert!(
                    compile_error.is_none(),
                    "line {line}: duplicate compile error"
                );
                compile_error = Some(directive_string(argument, line));
            }
            "contains" | "not-contains" | "count" | "min-count" | "in-order" => {}
            _ => panic!("line {line}: unknown Sonatina directive {command:?}"),
        }
    }
    assert!(has_directives, "missing //~ sonatina: expectations");
    assert!(
        compile_error.is_none() || !compile.is_empty(),
        "compile error requires a compile level"
    );
    let checks = TextChecks::from_directives(source, "sonatina");
    let source = source.to_owned();
    run_in_large_stack(move || {
        let (module, ir) = lower_source_inner(&source, file_url_imports);
        checks.assert_matches(&ir);
        for level in compile {
            let result = EvmCompile::new(module.clone_for_funcs(&module.funcs()))
                .with_opt_level(level)
                .compile();
            if let Some(expected) = &compile_error {
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
    });
}

#[test]
fn runtime_entry_calls_the_declared_function_and_deploys_its_result() {
    use solcore_test_utils::e2e::{
        e2e_enabled, e2e_required, encode_hex, parse_e2e_scenario, with_evm_harness,
    };

    run_in_large_stack(|| {
        let (module, ir) = lower_source(include_str!(
            "../fixtures/regressions/entry_identity/main.sol"
        ));
        let entry = ir
            .split_once("func public %solcore_entry_12_root_2eruntime")
            .expect("runtime entry")
            .1;
        assert!(entry.contains("main_5fmain_5fd"), "{entry}");
        assert!(!entry.contains("main_5fmain_5fa_5fd"), "{entry}");

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
        let scenario = parse_e2e_scenario(
            "OutputDeploy",
            [
                r#"#[scenario("entry_identity")]"#,
                "#[deploy(0x) -> success(0x000000000000000000000000000000000000000000000000000000000000002a)]",
            ],
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
    });
}
