// Expected behavior is expressed in Rust so fixture discovery only depends on source files.
use super::*;

pub(super) fn expectations(path: &str) -> Case {
    match path {
        "crates/hull/tests/fixtures/regressions/array_checked_typedef/main.sol" => Case {
            functions: BTreeMap::from([
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
            ]),
            ..Case::default()
        },
        "crates/hull/tests/fixtures/regressions/array_local_storage/main.sol" => Case {
            checks: TextChecks {
                contains: vec![
                    "LValueIdxAccess_lookup".into(),
                    "std_lidx".into(),
                    "std_out_of_bounds".into(),
                ],
                ..TextChecks::default()
            },
            ..Case::default()
        },
        "crates/hull/tests/fixtures/regressions/array_nested_dynamic/main.sol" => Case {
            checks: TextChecks {
                contains: vec![
                    "LValueIdxAccess_lookup".into(),
                    "storeBytesFromMemory".into(),
                    "loadBytesFromStorage".into(),
                    "frombool".into(),
                    "tobool".into(),
                ],
                ..TextChecks::default()
            },
            ..Case::default()
        },
        "crates/hull/tests/fixtures/regressions/assembly_function_no_capture/main.sol" => Case {
            loader: Loader::Memory,
            undefined_variable: Some("x".into()),
            ..Case::default()
        },
        "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/asm-match-tuple-write-read.sol"
        | "crates/hull/tests/fixtures/regressions/read_only_known_match_comptime/main.sol" => {
            Case {
                loader: Loader::Memory,
                ..Case::default()
            }
        }
        "crates/hull/tests/fixtures/regressions/constructor_arguments/main.sol" => Case {
            checks: TextChecks {
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
                            not_contains: vec![
                                "copy_arguments_for_constructor".into(),
                                "_start".into(),
                            ],
                            ..TextChecks::default()
                        },
                    },
                ],
                ..TextChecks::default()
            },
            ..Case::default()
        },
        "tests/e2e/control-flow-capture-identity/main.sol"
        | "tests/e2e/control-flow-local-lifetimes/main.sol"
        | "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/closure-free-var.sol"
        | "crates/hull/tests/fixtures/local-lambda-helper.sol"
        | "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/inc-closure.sol"
        | "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/closure-free-var-local.sol"
        | "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/SimpleLambda.sol"
        | "crates/parser/tests/fixtures/corpus/ok/test/examples/cases/closure-free-var-std.sol" => {
            Case { ..Case::default() }
        }
        "crates/parser/tests/fixtures/corpus/ok/test/examples/dispatch/empty_no_constructor.sol" => {
            Case {
                checks: TextChecks {
                    contains: vec![
                        "object \"CDeploy\"".into(),
                        "object \"C\"".into(),
                        "codecopy(0, dataoffset(\"C\"), size)".into(),
                    ],
                    ..TextChecks::default()
                },
                ..Case::default()
            }
        }
        "crates/hull/tests/fixtures/regressions/irrefutable_match_scope/main.sol" => Case {
            functions: BTreeMap::from([(
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
            )]),
            loader: Loader::Memory,
            ..Case::default()
        },
        "crates/parser/tests/fixtures/corpus/ok/test/examples/dispatch/nonpayable_ctor.sol" => {
            Case {
                checks: TextChecks {
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
                },
                ..Case::default()
            }
        }
        "crates/hull/tests/fixtures/regressions/nullary_constructor_overlay/main.sol" => Case {
            checks: TextChecks {
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
            },
            loader: Loader::Memory,
            ..Case::default()
        },
        "crates/parser/tests/fixtures/corpus/ok/test/examples/dispatch/payable_ctor.sol" => Case {
            checks: TextChecks {
                regions: vec![solcore_test_utils::text_checks::TextRegion {
                    after: None,
                    before: Some("object \"PayableCtor\" {".into()),
                    checks: TextChecks {
                        not_contains: vec!["0xb5988ea3".into()],
                        ..TextChecks::default()
                    },
                }],
                ..TextChecks::default()
            },
            ..Case::default()
        },
        "crates/hull/tests/fixtures/regressions/specialized_control_flow_locals/main.sol" => Case {
            checks: TextChecks {
                min_count: BTreeMap::from([("let result".into(), 2)]),
                not_contains: vec!["$if_local".into()],
                ..TextChecks::default()
            },
            ..Case::default()
        },
        _ => panic!("missing expectations for source fixture {path}"),
    }
}
