use super::*;
use dir_test::{Fixture, dir_test};
use solcore_test_utils::{run_in_large_stack, source_directives, text_checks::TextChecks};

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/regressions", glob: "**/main.sol")]
fn source_fixture(fixture: Fixture<&str>) {
    let source = fixture.content();
    let directives = source_directives(source, "hull").collect::<Vec<_>>();
    assert!(!directives.is_empty(), "Hull fixture needs expectations");
    let mut memory = None;
    let mut undefined = None;
    for (line, directive) in directives {
        let (command, value) = directive.split_once(' ').unwrap_or((directive, ""));
        match command {
            "valid" => assert!(value.is_empty(), "line {line}: unexpected argument"),
            "loader" => {
                assert!(memory.is_none(), "line {line}: duplicate loader");
                memory = Some(match value {
                    "memory" => true,
                    "fixture" => false,
                    _ => panic!("line {line}: unknown Hull loader {value}"),
                });
            }
            "undefined-variable" => {
                assert!(undefined.is_none(), "line {line}: duplicate diagnostic");
                assert!(!value.is_empty(), "line {line}: missing variable name");
                undefined = Some(value);
            }
            "contains" | "not-contains" | "count" | "min-count" | "in-order" => {}
            _ => panic!("line {line}: unknown Hull expectation {command}"),
        }
    }
    let hull = render_fixture(
        Path::new(fixture.path()),
        memory.unwrap_or(false),
        undefined,
    );
    TextChecks::from_directives(source, "hull").assert_matches(&hull);
}

fn render_fixture(path: &Path, memory: bool, undefined: Option<&str>) -> String {
    let path = path.to_owned();
    let undefined = undefined.map(str::to_owned);
    run_in_large_stack(move || {
        let (db, output) = if memory {
            specialize_src(
                "source_fixture",
                &fs::read_to_string(&path).expect("source"),
            )
        } else {
            specialize_fixture(&path)
        };
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let emitted = emit_module(db, &output.module, EmitOptions::default());
        assert!(emitted.diagnostics.is_empty(), "{:?}", emitted.diagnostics);
        let diagnostics = check_program_with_db(db, &emitted.program);
        if let Some(expected) = undefined {
            assert!(diagnostics.iter().any(|diagnostic| matches!(
                &diagnostic.kind, CheckDiagnosticKind::UndefinedVariable { name } if name == &expected
            )), "{diagnostics:?}");
        } else {
            assert!(diagnostics.is_empty(), "{diagnostics:?}");
        }
        pretty_program(db, &emitted.program)
    })
}

fn render_shared_fixture(path: &str, memory: bool) -> String {
    render_fixture(&repo_root().join(path), memory, None)
}

#[test]
fn checked_typedef_array_operations() {
    let hull = render_shared_fixture(
        "crates/hull/tests/fixtures/regressions/array_checked_typedef/main.sol",
        false,
    );
    let functions: BTreeMap<String, TextChecks> = BTreeMap::from([
        (
            "LValueIdxAccess_lookup".into(),
            TextChecks {
                contains: vec![
                    "opcodes_sload".into(),
                    "std_ge_".into(),
                    "std_out_of_bounds".into(),
                    "Typedef_rep".into(),
                    "Second".into(),
                    "std_hash1".into(),
                ],
                ..TextChecks::default()
            },
        ),
        (
            "__solcore_memory_array_index".into(),
            TextChecks {
                contains: vec![
                    "lt(index, mload(base))".into(),
                    "0xb4120f14".into(),
                    "mload(add(add(base, 32), mul(index, 32)))".into(),
                ],
                ..TextChecks::default()
            },
        ),
    ]);
    for (function, checks) in functions {
        checks.assert_matches(hull_function(&hull, &function));
    }
}

#[test]
fn constructor_argument_deployment() {
    let hull = render_shared_fixture(
        "crates/hull/tests/fixtures/regressions/constructor_arguments/main.sol",
        false,
    );
    (TextChecks {
        regions: vec![
            solcore_test_utils::text_checks::TextRegion {
                after: None,
                before: Some("object \"C\" {".into()),
                checks: TextChecks {
                    contains: vec![
                        "copy_arguments_for_constructor".into(),
                        "abi_decode".into(),
                        "MemoryWordReader".into(),
                        "argSize := sub(codesize(), programSize)".into(),
                        "mstore(memoryDataOffset, argSize)".into(),
                        "payloadOffset := add(memoryDataOffset, 32)".into(),
                        "codecopy(payloadOffset, programSize, argSize)".into(),
                    ],
                    min_count: BTreeMap::from([("_start".into(), 2)]),
                    not_contains: vec![
                        "minimumSize".into(),
                        "BoundedMemoryWordReader".into(),
                        "codecopy(0, datasize(\"CDeploy\"), 32)".into(),
                        "constructor_arg".into(),
                    ],
                    ..TextChecks::default()
                },
            },
            solcore_test_utils::text_checks::TextRegion {
                after: Some("object \"C\" {".into()),
                before: None,
                checks: TextChecks {
                    not_contains: vec!["copy_arguments_for_constructor".into(), "_start".into()],
                    ..TextChecks::default()
                },
            },
        ],
        ..TextChecks::default()
    })
    .assert_matches(&hull);
}

#[test]
fn irrefutable_match_owns_its_bindings() {
    let hull = render_shared_fixture(
        "crates/hull/tests/fixtures/regressions/irrefutable_match_scope/main.sol",
        true,
    );
    let functions: BTreeMap<String, TextChecks> = BTreeMap::from([(
        "_main_".into(),
        TextChecks {
            in_order: vec![
                "\n  {\n".into(),
                "let y".into(),
                "let x".into(),
                "mstore(128, x)".into(),
                "\n  }\n".into(),
                "return x".into(),
            ],
            ..TextChecks::default()
        },
    )]);
    for (function, checks) in functions {
        checks.assert_matches(hull_function(&hull, &function));
    }
}

#[test]
fn nullary_constructor_overlay() {
    let hull = render_shared_fixture(
        "crates/hull/tests/fixtures/regressions/nullary_constructor_overlay/main.sol",
        true,
    );
    (TextChecks {
        regions: vec![solcore_test_utils::text_checks::TextRegion {
            after: None,
            before: Some("object \"C\" {".into()),
            checks: TextChecks {
                contains: vec![
                    "_start".into(),
                    "init_".into(),
                    "codecopy(0, dataoffset(\"C\"), size)".into(),
                ],
                not_contains: vec!["constructor_arg".into()],
                ..TextChecks::default()
            },
        }],
        ..TextChecks::default()
    })
    .assert_matches(&hull);
}

#[test]
fn empty_constructor_deployment() {
    let hull = render_shared_fixture(
        "crates/parser/tests/fixtures/corpus/ok/test/examples/dispatch/empty_no_constructor.sol",
        false,
    );
    (TextChecks {
        contains: vec![
            "object \"CDeploy\"".into(),
            "object \"C\"".into(),
            "codecopy(0, dataoffset(\"C\"), size)".into(),
        ],
        ..TextChecks::default()
    })
    .assert_matches(&hull);
}

#[test]
fn nonpayable_constructor_deployment() {
    let hull = render_shared_fixture(
        "crates/parser/tests/fixtures/corpus/ok/test/examples/dispatch/nonpayable_ctor.sol",
        false,
    );
    (TextChecks {
        regions: vec![
            solcore_test_utils::text_checks::TextRegion {
                after: None,
                before: Some("object \"NonPayableCtor\" {".into()),
                checks: TextChecks {
                    contains: vec![
                        "object \"NonPayableCtorDeploy\"".into(),
                        "mstore(64, memoryguard(128))".into(),
                        "datasize(\"NonPayableCtorDeploy\")".into(),
                        "if callvalue()".into(),
                        "0xb5988ea3".into(),
                        "codecopy(0, dataoffset(\"NonPayableCtor\"), size)".into(),
                        "return(0, size)".into(),
                    ],
                    min_count: BTreeMap::from([("_start".into(), 2)]),
                    ..TextChecks::default()
                },
            },
            solcore_test_utils::text_checks::TextRegion {
                after: Some("object \"NonPayableCtor\" {".into()),
                before: None,
                checks: TextChecks {
                    not_contains: vec!["_start".into()],
                    ..TextChecks::default()
                },
            },
        ],
        ..TextChecks::default()
    })
    .assert_matches(&hull);
}

#[test]
fn payable_constructor_deployment() {
    let hull = render_shared_fixture(
        "crates/parser/tests/fixtures/corpus/ok/test/examples/dispatch/payable_ctor.sol",
        false,
    );
    (TextChecks {
        regions: vec![solcore_test_utils::text_checks::TextRegion {
            after: None,
            before: Some("object \"PayableCtor\" {".into()),
            checks: TextChecks {
                not_contains: vec!["0xb5988ea3".into()],
                ..TextChecks::default()
            },
        }],
        ..TextChecks::default()
    })
    .assert_matches(&hull);
}

#[test]
fn assembly_tuple_pattern_write() {
    render_shared_fixture(
        "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/asm-match-tuple-write-read.sol",
        true,
    );
}

#[test]
fn control_flow_capture_identity() {
    render_shared_fixture("tests/e2e/control-flow-capture-identity/main.sol", false);
}

#[test]
fn control_flow_local_lifetimes() {
    render_shared_fixture("tests/e2e/control-flow-local-lifetimes/main.sol", false);
}

#[test]
fn inferred_lambda_capture() {
    render_shared_fixture(
        "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/closure-free-var.sol",
        false,
    );
}

#[test]
fn inferred_lambda_helper() {
    render_shared_fixture("crates/hull/tests/fixtures/local-lambda-helper.sol", false);
}

#[test]
fn inferred_lambda_increment() {
    render_shared_fixture(
        "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/inc-closure.sol",
        false,
    );
}

#[test]
fn inferred_lambda_local_capture() {
    render_shared_fixture(
        "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/closure-free-var-local.sol",
        false,
    );
}

#[test]
fn inferred_lambda_simple() {
    render_shared_fixture(
        "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/SimpleLambda.sol",
        false,
    );
}

#[test]
fn inferred_lambda_std_capture() {
    render_shared_fixture(
        "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/closure-free-var-std.sol",
        false,
    );
}
