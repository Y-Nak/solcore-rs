//! Expectations for the editor buffers, checked by the Rust compiler.
use super::*;

pub(super) fn case(name: &str) -> Case {
    match name {
        "comments_do_not_change_call_delimiters_or_argument_index_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Signature {
                at: Cursor {
                    needle: "  2".into(),
                    ..Default::default()
                },
                active_parameter: 1,
            }],
        },
        "comments_do_not_change_call_delimiters_or_argument_index_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Signature {
                at: Cursor {
                    needle: "  2".into(),
                    ..Default::default()
                },
                active_parameter: 1,
            }],
        },
        "comments_do_not_change_call_delimiters_or_argument_index_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Signature {
                at: Cursor {
                    needle: "  2".into(),
                    ..Default::default()
                },
                active_parameter: 1,
            }],
        },
        "comments_do_not_change_call_delimiters_or_argument_index_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Signature {
                at: Cursor {
                    needle: "  2".into(),
                    ..Default::default()
                },
                active_parameter: 1,
            }],
        },
        "comments_do_not_change_call_delimiters_or_argument_index_5" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Signature {
                at: Cursor {
                    needle: "  2".into(),
                    ..Default::default()
                },
                active_parameter: 1,
            }],
        },
        "completion_respects_block_arm_lambda_and_initializer_scopes" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Completion {
                    at: Cursor {
                        needle: "return later".into(),
                        offset: 7,
                        ..Default::default()
                    },
                    contains: vec![
                        ExpectedCompletion {
                            label: "input".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                        ExpectedCompletion {
                            label: "later".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                    ],
                    excludes: vec![
                        "hidden".into(),
                        "arm".into(),
                        "armLocal".into(),
                        "nested".into(),
                    ],
                    parse_clean: true,
                },
                Check::Completion {
                    at: Cursor {
                        needle: "hidden; }".into(),
                        ..Default::default()
                    },
                    contains: vec![ExpectedCompletion {
                        label: "hidden".into(),
                        kind: CompletionItemKind::VARIABLE,
                    }],
                    parse_clean: false,
                    excludes: vec![],
                },
                Check::Completion {
                    at: Cursor {
                        needle: "arm; }".into(),
                        ..Default::default()
                    },
                    contains: vec![
                        ExpectedCompletion {
                            label: "arm".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                        ExpectedCompletion {
                            label: "armLocal".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                    ],
                    parse_clean: false,
                    excludes: vec![],
                },
                Check::Completion {
                    at: Cursor {
                        needle: "nested; return".into(),
                        ..Default::default()
                    },
                    contains: vec![
                        ExpectedCompletion {
                            label: "nested".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                        ExpectedCompletion {
                            label: "input".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                    ],
                    parse_clean: false,
                    excludes: vec![],
                },
                Check::Completion {
                    at: Cursor {
                        needle: "let later = input".into(),
                        offset: 12,
                        ..Default::default()
                    },
                    contains: vec![],
                    excludes: vec!["later".into()],
                    parse_clean: false,
                },
            ],
        },
        "completion_uses_body_scope_even_without_an_expression" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Completion {
                at: Cursor {
                    needle: "  \n".into(),
                    offset: 2,
                    ..Default::default()
                },
                contains: vec![ExpectedCompletion {
                    label: "input".into(),
                    kind: CompletionItemKind::VARIABLE,
                }],
                parse_clean: false,
                excludes: vec![],
            }],
        },
        "constructor_rename_follows_inference_across_imports_aliases_and_lambdas" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Pick".into(),
                        document: "math.sol".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "Selected".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected/1/main.sol.txt".into()),
                        ("math.sol".into(), "expected/1/math.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Pick".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "Selected".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected/2/main.sol.txt".into()),
                        ("math.sol".into(), "expected/2/math.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
            ],
        },
        "constructor_rename_includes_expected_type_expression_and_pattern_references_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Pick".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "Selected".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                references: Some(3),
                definition: Some(Cursor {
                    needle: "Pick".into(),
                    ..Default::default()
                }),
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "constructor_rename_includes_expected_type_expression_and_pattern_references_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Pick".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "Selected".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                references: Some(3),
                definition: Some(Cursor {
                    needle: "Pick".into(),
                    ..Default::default()
                }),
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "constructor_rename_includes_expected_type_expression_and_pattern_references_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Pick".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "Selected".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                references: Some(3),
                definition: Some(Cursor {
                    needle: "Pick".into(),
                    ..Default::default()
                }),
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "constructor_rename_includes_expected_type_expression_and_pattern_references_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Pick".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "Selected".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                references: Some(3),
                definition: Some(Cursor {
                    needle: "Pick".into(),
                    ..Default::default()
                }),
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "constructor_rename_preserves_generic_impl_and_method_contexts_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Pick".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "Selected".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "constructor_rename_preserves_generic_impl_and_method_contexts_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Pick".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "Selected".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "constructor_rename_preserves_generic_impl_and_method_contexts_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Pick".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "Selected".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "imported_rename_rejects_global_collisions_even_without_references_1" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        document: "math.sol".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        document: "math.sol".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "imported_rename_rejects_global_collisions_even_without_references_2" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        document: "math.sol".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        document: "math.sol".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "imported_rename_rejects_global_collisions_even_without_references_3" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        document: "math.sol".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        document: "math.sol".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "imported_rename_rejects_global_collisions_even_without_references_alias" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "target".into(),
                    document: "math.sol".into(),
                    occurrence: Occurrence::Last,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "lexical_control_flow_scopes" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Completion {
                    at: Cursor {
                        needle: "/* then */".into(),
                        ..Default::default()
                    },
                    parse_clean: true,
                    contains: vec![
                        ExpectedCompletion {
                            label: "outer".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                        ExpectedCompletion {
                            label: "branch".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                    ],
                    excludes: vec!["index".into(), "inner".into()],
                },
                Check::Completion {
                    at: Cursor {
                        needle: "/* else */".into(),
                        ..Default::default()
                    },
                    parse_clean: true,
                    contains: vec![ExpectedCompletion {
                        label: "outer".into(),
                        kind: CompletionItemKind::VARIABLE,
                    }],
                    excludes: vec!["branch".into(), "index".into(), "inner".into()],
                },
                Check::Completion {
                    at: Cursor {
                        needle: "/* loop */".into(),
                        ..Default::default()
                    },
                    parse_clean: true,
                    contains: vec![
                        ExpectedCompletion {
                            label: "outer".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                        ExpectedCompletion {
                            label: "index".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                        ExpectedCompletion {
                            label: "inner".into(),
                            kind: CompletionItemKind::VARIABLE,
                        },
                    ],
                    excludes: vec!["branch".into()],
                },
                Check::Completion {
                    at: Cursor {
                        needle: "/* after */".into(),
                        ..Default::default()
                    },
                    parse_clean: true,
                    contains: vec![ExpectedCompletion {
                        label: "outer".into(),
                        kind: CompletionItemKind::VARIABLE,
                    }],
                    excludes: vec!["branch".into(), "index".into(), "inner".into()],
                },
            ],
        },
        "local_rename_bounded_analysis" => Case {
            documents: vec!["main.sol".into(), "other.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value: word) returns (word) {\n    let local".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    prepare: true,
                    references: Some(3),
                    max_resolved_bodies: Some(4),
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected_value.sol.txt".into()),
                        ("other.sol".into(), "other.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "local = value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    prepare: true,
                    references: Some(4),
                    max_resolved_bodies: Some(4),
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected_local.sol.txt".into()),
                        ("other.sol".into(), "other.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "arg: word".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    prepare: true,
                    references: Some(2),
                    max_resolved_bodies: Some(4),
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected_arg.sol.txt".into()),
                        ("other.sol".into(), "other.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "pattern {".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    prepare: true,
                    references: Some(4),
                    max_resolved_bodies: Some(4),
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected_pattern.sol.txt".into()),
                        ("other.sol".into(), "other.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value: word) returns (word) {\n    let local".into(),
                        ..Default::default()
                    },
                    new_name: "local".into(),
                    accepted: false,
                    max_resolved_bodies: Some(4),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "local = value".into(),
                        ..Default::default()
                    },
                    new_name: "arg".into(),
                    accepted: false,
                    max_resolved_bodies: Some(4),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "pattern {".into(),
                        ..Default::default()
                    },
                    new_name: "local".into(),
                    accepted: false,
                    max_resolved_bodies: Some(4),
                    ..Default::default()
                }),
            ],
        },
        "qualified_completion_after_subtraction_resolves_the_right_operand" => Case {
            documents: vec!["classes.sol".into(), "main.sol".into()],
            checks: vec![Check::Completion {
                at: Cursor {
                    needle: "Eq.".into(),
                    offset: 3,
                    ..Default::default()
                },
                contains: vec![ExpectedCompletion {
                    label: "eq".into(),
                    kind: CompletionItemKind::METHOD,
                }],
                excludes: vec!["Eq.eq".into()],
                parse_clean: false,
            }],
        },
        "rename_assembly_reference_ignores_locals_in_separate_scopes" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_assembly_reference_keeps_source_shadowing_bindings_separate" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "pop(value)".into(),
                    offset: 4,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_assembly_reference_rejects_capture_by_a_yul_local_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_assembly_reference_rejects_capture_by_a_yul_local_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_assembly_reference_rejects_capture_by_a_yul_local_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_assembly_references_preserves_yul_scopes_and_namespaces" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value:".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    expected_documents: BTreeMap::from([(
                        "main.sol".into(),
                        "expected/1/main.sol.txt".into(),
                    )]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "pop(value)".into(),
                        offset: 4,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    expected_documents: BTreeMap::from([(
                        "main.sol".into(),
                        "expected/2/main.sol.txt".into(),
                    )]),
                    ..Default::default()
                }),
            ],
        },
        "rename_cannot_capture_an_imported_module_qualifier" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_constructor_cannot_capture_an_unedited_pattern_binder_1" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Pick".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "value".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Pick".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "Renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_constructor_cannot_capture_an_unedited_pattern_binder_2" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Pick".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "value".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Pick".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "Renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_constructor_respects_hidden_and_contract_local_visibility_1" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Pick".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Pick".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "value".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_constructor_respects_hidden_and_contract_local_visibility_2" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Pick".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Pick".into(),
                        ..Default::default()
                    },
                    new_name: "value".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_function_preserves_initializer_and_disjoint_scope_lookup_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "target".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_function_preserves_initializer_and_disjoint_scope_lookup_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "target".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_function_preserves_initializer_and_disjoint_scope_lookup_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "target".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_imported_items_and_aliases_rejects_capture_at_consumer_1" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_imported_items_and_aliases_rejects_capture_at_consumer_2" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "alias".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "alias".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_imported_items_and_aliases_rejects_capture_at_consumer_3" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "M".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "M".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_items_and_fields_rejects_capture_by_source_bindings_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_items_and_fields_rejects_capture_by_source_bindings_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_items_and_fields_rejects_capture_by_source_bindings_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "target".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_pattern_binder_observes_imported_and_contract_constructor_visibility_1" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Choice".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Other".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_pattern_binder_observes_imported_and_contract_constructor_visibility_2" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Choice".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Other".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_pattern_binder_preserves_its_role_in_every_pattern_form_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Choice".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Other".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_pattern_binder_preserves_its_role_in_every_pattern_form_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Choice".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Other".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_pattern_binder_preserves_its_role_in_every_pattern_form_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Choice".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "Other".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_pattern_validation_maps_unicode_and_length_changing_edits" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::RenameSequence {
                steps: vec![
                    Rename {
                        at: Cursor {
                            needle: "fλ".into(),
                            ..Default::default()
                        },
                        new_name: "renamed漢".into(),
                        accepted: true,
                        no_errors_before: true,
                        no_errors_after: true,
                        expected_documents: BTreeMap::from([(
                            "main.sol".into(),
                            "expected/1/main.sol.txt".into(),
                        )]),
                        ..Default::default()
                    },
                    Rename {
                        at: Cursor {
                            needle: "value".into(),
                            ..Default::default()
                        },
                        new_name: "bindingλ".into(),
                        accepted: true,
                        no_errors_before: true,
                        no_errors_after: true,
                        expected_documents: BTreeMap::from([(
                            "main.sol".into(),
                            "expected/2/main.sol.txt".into(),
                        )]),
                        ..Default::default()
                    },
                ],
            }],
        },
        "rename_preserves_initializer_and_disjoint_scope_lookup_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_preserves_initializer_and_disjoint_scope_lookup_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_preserves_initializer_and_disjoint_scope_lookup_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_preserves_initializer_and_disjoint_scope_lookup_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_preserves_initializer_and_disjoint_scope_lookup_5" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_preserves_initializer_and_disjoint_scope_lookup_6" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_preserves_initializer_and_disjoint_scope_lookup_7" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_names_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "comptime".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_names_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "comptime".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_names_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "comptime".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_names_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "comptime".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_names_5" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "while".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_names_6" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "mapping".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_names_7" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "derive".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_type_and_value_names_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Original".into(),
                    ..Default::default()
                },
                new_name: "comptime".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_type_and_value_names_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Original".into(),
                    ..Default::default()
                },
                new_name: "comptime".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_type_and_value_names_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Original".into(),
                    ..Default::default()
                },
                new_name: "mapping".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_type_and_value_names_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Original".into(),
                    ..Default::default()
                },
                new_name: "mapping".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_type_and_value_names_5" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Original".into(),
                    ..Default::default()
                },
                new_name: "mapping".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_type_and_value_names_6" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Original".into(),
                    ..Default::default()
                },
                new_name: "comptime".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_type_and_value_names_7" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Original".into(),
                    ..Default::default()
                },
                new_name: "mapping".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                ..Default::default()
            })],
        },
        "rename_preserves_legal_contextual_type_and_value_names_aliased" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: true,
                    no_errors_after: true,
                    contents: vec![ContentExpectation {
                        text: "Alias<word>".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: true,
                    no_errors_after: true,
                    contents: vec![ContentExpectation {
                        text: "Alias<word>".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            ],
        },
        "rename_preserves_legal_contextual_type_and_value_names_qualified" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    contents: vec![ContentExpectation {
                        text: "M.mapping<word>".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    contents: vec![ContentExpectation {
                        text: "M.mapping<word>".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    contents: vec![ContentExpectation {
                        text: "M.comptime<word>".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    contents: vec![ContentExpectation {
                        text: "M.comptime<word>".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            ],
        },
        "rename_preserves_unrelated_existing_errors" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    ..Default::default()
                },
                new_name: "longerName".into(),
                accepted: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_rejects_capture_of_unedited_source_and_assembly_references_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_capture_of_unedited_source_and_assembly_references_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_capture_of_unedited_source_and_assembly_references_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_capture_of_unedited_source_and_assembly_references_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_capture_of_unedited_source_and_assembly_references_5" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_capture_of_unedited_source_and_assembly_references_6" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_capture_of_unedited_source_and_assembly_references_7" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_capture_of_unedited_source_and_assembly_references_8" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_contextual_type_syntax_at_unqualified_references_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_contextual_type_syntax_at_unqualified_references_10" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_contextual_type_syntax_at_unqualified_references_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_contextual_type_syntax_at_unqualified_references_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_contextual_type_syntax_at_unqualified_references_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_contextual_type_syntax_at_unqualified_references_5" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_contextual_type_syntax_at_unqualified_references_6" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_contextual_type_syntax_at_unqualified_references_7" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_contextual_type_syntax_at_unqualified_references_8" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_contextual_type_syntax_at_unqualified_references_9" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_global_capture_of_untouched_references_and_imported_type_capture_global" => {
            Case {
                documents: vec!["main.sol".into(), "math.sol".into()],
                checks: vec![Check::Rename(Rename {
                    at: Cursor {
                        needle: "local".into(),
                        ..Default::default()
                    },
                    new_name: "value".into(),
                    accepted: false,
                    ..Default::default()
                })],
            }
        }
        "rename_rejects_global_capture_of_untouched_references_and_imported_type_capture_type_1" => {
            Case {
                documents: vec!["main.sol".into(), "math.sol".into()],
                checks: vec![
                    Check::Rename(Rename {
                        at: Cursor {
                            needle: "Target".into(),
                            document: "math.sol".into(),
                            occurrence: Occurrence::Last,
                            ..Default::default()
                        },
                        new_name: "a".into(),
                        accepted: false,
                        no_errors_before: true,
                        ..Default::default()
                    }),
                    Check::Rename(Rename {
                        at: Cursor {
                            needle: "Target".into(),
                            document: "math.sol".into(),
                            occurrence: Occurrence::Last,
                            ..Default::default()
                        },
                        new_name: "SafeType".into(),
                        accepted: true,
                        no_errors_before: true,
                        ..Default::default()
                    }),
                ],
            }
        }
        "rename_rejects_global_capture_of_untouched_references_and_imported_type_capture_type_2" => {
            Case {
                documents: vec!["main.sol".into(), "math.sol".into()],
                checks: vec![
                    Check::Rename(Rename {
                        at: Cursor {
                            needle: "Kept".into(),
                            ..Default::default()
                        },
                        new_name: "a".into(),
                        accepted: false,
                        no_errors_before: true,
                        ..Default::default()
                    }),
                    Check::Rename(Rename {
                        at: Cursor {
                            needle: "Kept".into(),
                            ..Default::default()
                        },
                        new_name: "SafeType".into(),
                        accepted: true,
                        no_errors_before: true,
                        ..Default::default()
                    }),
                ],
            }
        }
        "rename_rejects_source_bindings_that_capture_edited_references_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_source_bindings_that_capture_edited_references_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_source_bindings_that_capture_edited_references_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_source_bindings_that_capture_edited_references_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_source_bindings_that_capture_edited_references_5" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_source_bindings_that_capture_edited_references_6" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_rejects_source_bindings_that_capture_edited_references_7" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::All,
                        ..Default::default()
                    },
                    new_name: "safeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_respects_runtime_parameter_contexts_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_respects_runtime_parameter_contexts_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_respects_runtime_parameter_contexts_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_respects_runtime_parameter_contexts_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_respects_runtime_parameter_contexts_5" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_respects_runtime_parameter_contexts_6" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "value".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_source_bindings_includes_assembly_reads_and_assignments_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_source_bindings_includes_assembly_reads_and_assignments_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_source_bindings_includes_assembly_reads_and_assignments_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_source_bindings_includes_assembly_reads_and_assignments_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_source_bindings_includes_assembly_reads_and_assignments_5" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_source_bindings_includes_assembly_reads_and_assignments_6" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_source_bindings_includes_assembly_reads_and_assignments_7" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_source_bindings_includes_assembly_reads_and_assignments_8" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_source_bindings_includes_assembly_reads_and_assignments_9" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "value".into(),
                    occurrence: Occurrence::All,
                    ..Default::default()
                },
                new_name: "renamed".into(),
                accepted: true,
                no_errors_before: true,
                no_errors_after: true,
                prepare: true,
                expected_documents: BTreeMap::from([(
                    "main.sol".into(),
                    "expected/1/main.sol.txt".into(),
                )]),
                ..Default::default()
            })],
        },
        "rename_type_preserves_unqualified_constructor_pattern_roles" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![Check::Rename(Rename {
                at: Cursor {
                    needle: "Choice".into(),
                    ..Default::default()
                },
                new_name: "Renamed".into(),
                accepted: false,
                no_errors_before: true,
                ..Default::default()
            })],
        },
        "rename_validates_imported_type_alias_uses_and_unqualified_consumers_1" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Original".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
            ],
        },
        "rename_validates_imported_type_alias_uses_and_unqualified_consumers_2" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Box".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Box".into(),
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Box".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Box".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "comptime".into(),
                    accepted: false,
                    no_errors_before: true,
                    prepare: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Box".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "mapping".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "renaming_hidden_import_updates_provider_and_hiding_occurrence" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "hidden".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_after: true,
                    contents: vec![
                        ContentExpectation {
                            text: "hiding {renamed}".into(),
                            ..Default::default()
                        },
                        ContentExpectation {
                            text: "{renamed as kept}".into(),
                            ..Default::default()
                        },
                        ContentExpectation {
                            text: "hidden".into(),
                            document: "math.sol".into(),
                            absent: true,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "hidden".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_after: true,
                    contents: vec![
                        ContentExpectation {
                            text: "hiding {renamed}".into(),
                            ..Default::default()
                        },
                        ContentExpectation {
                            text: "{renamed as kept}".into(),
                            ..Default::default()
                        },
                        ContentExpectation {
                            text: "hidden".into(),
                            document: "math.sol".into(),
                            absent: true,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
            ],
        },
        "renaming_hidden_selected_import_updates_every_provider_reference_1" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "hidden".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    prepare: true,
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected/1/main.sol.txt".into()),
                        ("math.sol".into(), "expected/1/math.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "hidden".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    prepare: true,
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected/2/main.sol.txt".into()),
                        ("math.sol".into(), "expected/2/math.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "hidden".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    prepare: true,
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected/3/main.sol.txt".into()),
                        ("math.sol".into(), "expected/3/math.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
            ],
        },
        "renaming_hidden_selected_import_updates_every_provider_reference_2" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "hidden".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    prepare: true,
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected/1/main.sol.txt".into()),
                        ("math.sol".into(), "expected/1/math.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "hidden".into(),
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    prepare: true,
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected/2/main.sol.txt".into()),
                        ("math.sol".into(), "expected/2/math.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "hidden".into(),
                        occurrence: Occurrence::Last,
                        ..Default::default()
                    },
                    new_name: "renamed".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    prepare: true,
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected/3/main.sol.txt".into()),
                        ("math.sol".into(), "expected/3/math.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
            ],
        },
        "renaming_hidden_type_import_preserves_its_alias" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Choice".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "Option".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected/1/main.sol.txt".into()),
                        ("math.sol".into(), "expected/1/math.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Choice".into(),
                        ..Default::default()
                    },
                    new_name: "Option".into(),
                    accepted: true,
                    no_errors_before: true,
                    no_errors_after: true,
                    expected_documents: BTreeMap::from([
                        ("main.sol".into(), "expected/2/main.sol.txt".into()),
                        ("math.sol".into(), "expected/2/math.sol.txt".into()),
                    ]),
                    ..Default::default()
                }),
            ],
        },
        "renaming_trait_method_updates_all_impls_with_the_same_trait_identity" => Case {
            documents: vec!["main.sol".into(), "math.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "echo".into(),
                        document: "math.sol".into(),
                        ..Default::default()
                    },
                    new_name: "apply".into(),
                    accepted: true,
                    no_errors_after: true,
                    contents: vec![
                        ContentExpectation {
                            text: "function apply".into(),
                            count: Some(2),
                            ..Default::default()
                        },
                        ContentExpectation {
                            text: "function echo".into(),
                            count: Some(2),
                            ..Default::default()
                        },
                        ContentExpectation {
                            text: "Echo.apply(x)".into(),
                            ..Default::default()
                        },
                        ContentExpectation {
                            text: "function apply".into(),
                            document: "math.sol".into(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "echo".into(),
                        ..Default::default()
                    },
                    new_name: "apply".into(),
                    accepted: true,
                    no_errors_after: true,
                    contents: vec![
                        ContentExpectation {
                            text: "function apply".into(),
                            count: Some(2),
                            ..Default::default()
                        },
                        ContentExpectation {
                            text: "function echo".into(),
                            count: Some(2),
                            ..Default::default()
                        },
                        ContentExpectation {
                            text: "Echo.apply(x)".into(),
                            ..Default::default()
                        },
                        ContentExpectation {
                            text: "function apply".into(),
                            document: "math.sol".into(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
            ],
        },
        "type_rename_preserves_type_parameter_and_alias_bindings_1" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Target".into(),
                        ..Default::default()
                    },
                    new_name: "a".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Target".into(),
                        ..Default::default()
                    },
                    new_name: "SafeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "type_rename_preserves_type_parameter_and_alias_bindings_2" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Target".into(),
                        ..Default::default()
                    },
                    new_name: "a".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Target".into(),
                        ..Default::default()
                    },
                    new_name: "SafeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "type_rename_preserves_type_parameter_and_alias_bindings_3" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Target".into(),
                        ..Default::default()
                    },
                    new_name: "a".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Target".into(),
                        ..Default::default()
                    },
                    new_name: "SafeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "type_rename_preserves_type_parameter_and_alias_bindings_4" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Target".into(),
                        ..Default::default()
                    },
                    new_name: "a".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Target".into(),
                        ..Default::default()
                    },
                    new_name: "SafeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "type_rename_preserves_type_parameter_and_alias_bindings_5" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Target".into(),
                        ..Default::default()
                    },
                    new_name: "a".into(),
                    accepted: false,
                    no_errors_before: true,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "Target".into(),
                        ..Default::default()
                    },
                    new_name: "SafeName".into(),
                    accepted: true,
                    no_errors_before: true,
                    ..Default::default()
                }),
            ],
        },
        "type_rename_preserves_type_parameter_and_alias_bindings_parameter" => Case {
            documents: vec!["main.sol".into()],
            checks: vec![
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "<a>".into(),
                        offset: 1,
                        ..Default::default()
                    },
                    new_name: "Target".into(),
                    accepted: false,
                    ..Default::default()
                }),
                Check::Rename(Rename {
                    at: Cursor {
                        needle: "<a>".into(),
                        offset: 1,
                        ..Default::default()
                    },
                    new_name: "b".into(),
                    accepted: true,
                    ..Default::default()
                }),
            ],
        },
        _ => panic!("missing editor checks for {name}"),
    }
}
