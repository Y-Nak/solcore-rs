// Expected behavior is expressed in Rust so fixture discovery only depends on source files.
use super::*;

pub(super) fn expectations(path: &str) -> Case {
    match path {
        "tests/e2e/aggregate-closure-captures/main.sol" => Case {
            checks: vec![
                Check::NoClosureDispatch,
                Check::FunctionCount {
                    selector: Selector {
                        contains: vec!["repeatBox".into(), "$closure".into()],
                        ..Selector::default()
                    },
                    count: Some(1),
                    min: None,
                    max: None,
                },
                Check::FunctionCount {
                    selector: Selector {
                        contains: vec!["repeatPayload".into(), "$closure".into()],
                        ..Selector::default()
                    },
                    count: Some(1),
                    min: None,
                    max: None,
                },
            ],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/audit/call_materialization_keeps_unknown_comptime_obligations/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/call_materialization_keeps_unknown_comptime_obligations/002/main.invalid.sol.txt" => {
            Case {
                checks: vec![],
                diagnostics: vec![DiagnosticCheck {
                    kind: DiagnosticKind::ComptimeEvaluationFailed,
                    context: None,
                    context_contains: None,
                }],
                frontend_clean: Some(false),
                loader: Loader::StdCurrent,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/capture_type_scope/after_block/main.sol"
        | "crates/specialize/tests/fixtures/audit/capture_type_scope/after_for/main.sol"
        | "crates/specialize/tests/fixtures/audit/capture_type_scope/after_if/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/callable_capture/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/caller_capture_name_collision/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/caller_capture_uninitialized_name_collision/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/mixed_dead_unavailable_core_caller_alias/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/mixed_dead_unavailable_yul_caller_alias/main.sol" => {
            Case {
                checks: vec![Check::NoClosureDispatch],
                frontend_clean: Some(true),
                loader: Loader::Fixture,
                ..Case::default()
            }
        }
        "tests/e2e/comptime-pattern-scopes/main.sol" => Case {
            checks: vec![],
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/audit/constant_conditionals_preserve_outer_bindings_across_skipped_branches/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_preserve_outer_bindings_across_skipped_branches/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_preserve_outer_bindings_across_skipped_branches/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_preserve_outer_bindings_across_skipped_branches/004/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_preserve_outer_bindings_across_skipped_branches/005/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_preserve_outer_bindings_across_skipped_branches/006/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_preserve_outer_bindings_across_skipped_branches/007/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_preserve_outer_bindings_across_skipped_branches/008/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_preserve_outer_bindings_across_skipped_branches/009/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_preserve_outer_bindings_across_skipped_branches/010/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_declarations_preserve_outer_constants/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_declarations_preserve_outer_constants/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_declarations_preserve_outer_constants/003/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("1".into()),
                }],
                frontend_clean: Some(true),
                loader: Loader::Fixture,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/constant_conditionals_still_fold_when_discarded_bindings_are_lexical/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_still_fold_when_discarded_bindings_are_lexical/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_still_fold_when_discarded_bindings_are_lexical/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_still_fold_when_discarded_bindings_are_lexical/004/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_still_fold_when_discarded_bindings_are_lexical/005/main.sol"
        | "crates/specialize/tests/fixtures/audit/constant_conditionals_still_fold_when_discarded_bindings_are_lexical/006/main.sol"
        | "crates/specialize/tests/fixtures/audit/cyclic_superclass_dictionaries_preserve_repeated_projections/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/002/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("1".into()),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/control_flow_declarations_preserve_outer_constants/004/main.sol" => {
            Case {
                checks: vec![
                    Check::LoopConditionArgument { name: "i".into() },
                    Check::MainReturnNumber {
                        value: Some("1".into()),
                    },
                ],
                frontend_clean: Some(true),
                loader: Loader::Std,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/constant_if/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/dynamic_if/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/else_declaration/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/for_body/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/for_init/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/known_match/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/known_selected_shadow/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/nested_if/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/nested_scope/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/unknown_match/main.sol"
        | "crates/specialize/tests/fixtures/module_local_function_evidence/phantom.sol" => Case {
            checks: vec![Check::MainReturnNumber {
                value: Some("7".into()),
            }],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/initializer_read/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/loop_body_initializer_read/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/loop_initializer_read/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/prefix_read/main.sol" => {
            Case {
                checks: vec![Check::MainReturnVariable {
                    name: "result".into(),
                }],
                frontend_clean: Some(true),
                loader: Loader::Fixture,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/known_live_write/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/loop_prior_write/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/write_before_if/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("9".into()),
                }],
                frontend_clean: Some(true),
                loader: Loader::Fixture,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/loop_write_before_shadow/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/write_after_shadow/main.sol"
        | "crates/specialize/tests/fixtures/audit/control_flow_scope_restoration/write_before_shadow/main.sol" => {
            Case {
                checks: vec![Check::MainReturnVariable {
                    name: "seed".into(),
                }],
                frontend_clean: Some(true),
                loader: Loader::Fixture,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/cyclic_superclass_projections_reuse_recursive_function_specializations/001/main.sol" => {
            Case {
                checks: vec![Check::FunctionCount {
                    selector: Selector::default(),
                    count: None,
                    min: None,
                    max: Some(9),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/derived_method_constraints_adapt_nested_self_and_secondary_arguments/001/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("7".into()),
                }],
                frontend_clean: Some(true),
                loader: Loader::Std,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/derived_method_constraints_do_not_relabel_the_source_dictionary/001/main.sol" => {
            Case {
                checks: vec![],
                diagnostics: vec![DiagnosticCheck {
                    kind: DiagnosticKind::MissingEvidence,
                    context: Some("derived f: word: P".into()),
                    context_contains: None,
                }],
                frontend_clean: Some(true),
                loader: Loader::Std,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/derived_method_constraints_replace_self_binders_without_replacing_equal_method_arguments/001/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("106".into()),
                }],
                frontend_clean: Some(true),
                loader: Loader::Std,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/derived_methods_adapt_self_dependent_constraints/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/derived_methods_adapt_self_dependent_constraints/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/derived_methods_adapt_self_dependent_constraints/003/main.sol" => {
            Case {
                checks: vec![],
                frontend_clean: Some(true),
                loader: Loader::Std,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/destructured_comptime_generic_arguments_keep_trait_call_obligations/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/004/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/007/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/008/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/009/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/010/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/011/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/012/main.sol"
        | "crates/specialize/tests/fixtures/audit/match_bindings_inherit_only_their_own_scrutinee_availability/001/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("3".into()),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/destructured_comptime_generic_arguments_keep_trait_call_obligations/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/match_bindings_inherit_only_their_own_scrutinee_availability/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/match_bindings_inherit_only_their_own_scrutinee_availability/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/runtime_pattern_shadow_does_not_inherit_outer_comptime_availability/001/main.sol" => {
            Case {
                checks: vec![],
                diagnostics: vec![DiagnosticCheck {
                    kind: DiagnosticKind::ComptimeEvaluationFailed,
                    context: None,
                    context_contains: Some("runtime value passed to comptime parameter".into()),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/destructured_comptime_parameters_remain_available_to_comptime_calls/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/destructured_comptime_parameters_remain_available_to_comptime_calls/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/destructured_comptime_parameters_remain_available_to_comptime_calls/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/destructured_comptime_parameters_remain_available_to_comptime_calls/004/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("3".into()),
                }],
                loader: Loader::Std,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/005/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/006/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("6".into()),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/013/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/014/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("5".into()),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/015/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/016/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("2".into()),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/017/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/018/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/019/main.sol"
        | "crates/specialize/tests/fixtures/audit/folded_match_and_block_scopes_preserve_outer_bindings_and_writes/020/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("8".into()),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/folded_match_captures_closures_before_typed_pattern_shadowing/001/main.sol" => {
            Case {
                checks: vec![Check::NoClosureDispatch],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/generic_abi_decoder_evidence_specializes_for_internal_sum_adt/001/main.sol" => {
            Case {
                checks: vec![
                    Check::FunctionCount {
                        selector: Selector {
                            prefix: Some("Generic_to_d".into()),
                            ..Selector::default()
                        },
                        count: None,
                        min: Some(1),
                        max: None,
                    },
                    Check::FunctionCount {
                        selector: Selector {
                            prefix: Some("ABIDecode_decode_d".into()),
                            ..Selector::default()
                        },
                        count: None,
                        min: Some(1),
                        max: None,
                    },
                ],
                loader: Loader::Std,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/implicit_superclass_bindings_survive_recursive_instance_methods/001/main.sol" => {
            Case {
                checks: vec![Check::FunctionCount {
                    selector: Selector::default(),
                    count: None,
                    min: None,
                    max: Some(9),
                }],
                loader: Loader::Bare,
                max_depth: Some(16),
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/known_if_binding_order/dead_nested_binding_restore/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/direct_selected_argument/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/known_fallthrough_independent/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/known_return_argument/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/known_return_closure/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/known_return_comptime/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/return_before_later_shadow/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/skipped_initializer_effect/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/skipped_loop_restore/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/skipped_unused_callable_aggregate/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/skipped_unused_closure/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/skipped_unused_comptime/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/unavailable_dead_capture/main.sol"
        | "crates/specialize/tests/fixtures/audit/known_if_binding_order/unavailable_dead_capture_assembly/main.sol" => {
            Case {
                checks: vec![
                    Check::NoClosureDispatch,
                    Check::MainReturnNumber {
                        value: Some("7".into()),
                    },
                ],
                frontend_clean: Some(true),
                loader: Loader::Fixture,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/known_if_binding_order/direct_selected_comptime/main.sol" => {
            Case {
                checks: vec![
                    Check::NoClosureDispatch,
                    Check::MainReturnNumber {
                        value: Some("1".into()),
                    },
                ],
                frontend_clean: Some(true),
                loader: Loader::Fixture,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/known_if_binding_order/fallthrough_restore_latest_outer/main.sol" => {
            Case {
                checks: vec![
                    Check::NoClosureDispatch,
                    Check::MainReturnNumber {
                        value: Some("9".into()),
                    },
                ],
                frontend_clean: Some(true),
                loader: Loader::Fixture,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/lexical_control_flow_scopes/comptime_call/main.sol"
        | "crates/specialize/tests/fixtures/audit/lexical_control_flow_scopes/else_branch/main.sol"
        | "crates/specialize/tests/fixtures/audit/lexical_control_flow_scopes/lambda_capture/main.sol"
        | "crates/specialize/tests/fixtures/audit/lexical_control_flow_scopes/loop_initializer/main.sol"
        | "crates/specialize/tests/fixtures/audit/lexical_control_flow_scopes/reported/main.sol"
        | "crates/specialize/tests/fixtures/audit/lexical_control_flow_scopes/runtime_branches/main.sol"
        | "crates/specialize/tests/fixtures/audit/lexical_control_flow_scopes/while_body/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("1".into()),
                }],
                frontend_clean: Some(true),
                loader: Loader::Std,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/lexical_control_flow_scopes/outer_write/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("9".into()),
                }],
                frontend_clean: Some(true),
                loader: Loader::Std,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/loop_shadowing_preserves_outer_closure_captures/001/main.sol" => {
            Case {
                checks: vec![Check::FunctionParameters {
                    names: vec!["x".into()],
                    selector: Selector {
                        prefix: Some("__lambda".into()),
                        ..Selector::default()
                    },
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/loops_preserve_unmodified_comptime_values_and_callables/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/loops_preserve_unmodified_comptime_values_and_callables/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/loops_preserve_unmodified_comptime_values_and_callables/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/loops_preserve_unmodified_comptime_values_and_callables/004/main.sol"
        | "crates/specialize/tests/fixtures/audit/loops_preserve_unmodified_comptime_values_and_callables/005/main.sol"
        | "crates/specialize/tests/fixtures/audit/loops_preserve_unmodified_comptime_values_and_callables/006/main.sol"
        | "crates/specialize/tests/fixtures/audit/loops_preserve_unmodified_comptime_values_and_callables/007/main.sol"
        | "crates/specialize/tests/fixtures/audit/loops_preserve_unmodified_comptime_values_and_callables/008/main.sol"
        | "crates/specialize/tests/fixtures/audit/loops_preserve_unmodified_comptime_values_and_callables/009/main.sol"
        | "crates/specialize/tests/fixtures/audit/loops_preserve_unmodified_comptime_values_and_callables/010/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_match_assignments_preserve_shadowed_static_bindings/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_match_assignments_preserve_shadowed_static_bindings/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_match_assignments_preserve_shadowed_static_bindings/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_match_assignments_preserve_shadowed_static_bindings/004/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_match_assignments_preserve_shadowed_static_bindings/005/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_match_assignments_preserve_shadowed_static_bindings/006/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_match_assignments_preserve_shadowed_static_bindings/007/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_match_assignments_preserve_shadowed_static_bindings/008/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_match_assignments_preserve_shadowed_static_bindings/009/main.sol"
        | "crates/specialize/tests/fixtures/audit/shared_state_effects_do_not_invalidate_same_named_static_locals/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/shared_state_effects_do_not_invalidate_same_named_static_locals/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/shared_state_effects_do_not_invalidate_same_named_static_locals/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/shared_state_effects_do_not_invalidate_same_named_static_locals/004/main.sol"
        | "crates/specialize/tests/fixtures/audit/shared_state_effects_do_not_invalidate_same_named_static_locals/005/main.sol"
        | "crates/specialize/tests/fixtures/audit/shared_state_effects_do_not_invalidate_same_named_static_locals/006/main.sol"
        | "crates/specialize/tests/fixtures/audit/shared_state_effects_do_not_invalidate_same_named_static_locals/007/main.sol"
        | "crates/specialize/tests/fixtures/audit/shared_state_effects_do_not_invalidate_same_named_static_locals/008/main.sol"
        | "crates/specialize/tests/fixtures/audit/unknown_callees_preserve_caller_local_static_values/001/main.sol" => {
            Case {
                checks: vec![Check::EntryLastReturnNumber { value: "7".into() }],
                frontend_clean: Some(true),
                loader: Loader::StdCurrent,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/mixed_matches_specialize_static_columns_without_leaking_pattern_bindings/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/mixed_matches_specialize_static_columns_without_leaking_pattern_bindings/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/mixed_matches_specialize_static_columns_without_leaking_pattern_bindings/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/mixed_matches_specialize_static_columns_without_leaking_pattern_bindings/004/main.sol"
        | "crates/specialize/tests/fixtures/audit/mixed_matches_specialize_static_columns_without_leaking_pattern_bindings/005/main.sol" => {
            Case {
                checks: vec![Check::ResidualMatch {
                    arms: 1,
                    patterns: 1,
                    scrutinees: 1,
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/partial_pattern_specialization_preserves_explicit_staging_requirements/001/main.sol" => {
            Case {
                checks: vec![],
                diagnostics: vec![DiagnosticCheck {
                    kind: DiagnosticKind::IntegerErasure,
                    context: None,
                    context_contains: None,
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/public_dispatch_accepts_nine_parameter_functions/001/main.sol" => {
            Case {
                checks: vec![Check::EntryExists],
                frontend_clean: Some(true),
                loader: Loader::Std,
                ..Case::default()
            }
        }
        "tests/e2e/abi-word-memory/main.sol"
        | "tests/e2e/abi-word-custom-codec/main.sol"
        | "tests/e2e/abi-byte-memory/main.sol"
        | "tests/e2e/abi-raw-default-codec/main.sol" => Case {
            checks: vec![Check::EntryExists],
            frontend_clean: Some(true),
            loader: Loader::StdCurrent,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/audit/recursive_inferred_callable_signature_is_diagnosed/001/main.sol" => {
            Case {
                diagnostics: vec![DiagnosticCheck {
                    kind: DiagnosticKind::FreeTypeVariable,
                    context: None,
                    context_contains: None,
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/repeated_pattern_names_residualize_only_the_visible_declaration/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/repeated_pattern_names_residualize_only_the_visible_declaration/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/repeated_pattern_names_residualize_only_the_visible_declaration/003/main.sol" => {
            Case {
                checks: vec![Check::Binding {
                    count: Some(1),
                    min: None,
                    builtin: Some("bool".into()),
                    name: "value".into(),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/residual_calls_use_materialized_argument_types/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_calls_use_materialized_argument_types/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_calls_use_materialized_argument_types/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_calls_use_materialized_argument_types/004/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_calls_use_materialized_argument_types/005/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_calls_use_materialized_argument_types/006/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_calls_use_materialized_argument_types/007/main.sol"
        | "crates/specialize/tests/fixtures/audit/residual_calls_use_materialized_argument_types/008/main.sol" => {
            Case {
                checks: vec![Check::MaterializedReturnCall {
                    literal_argument: "7".into(),
                }],
                frontend_clean: Some(true),
                loader: Loader::StdCurrent,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/residual_match_still_invalidates_assignments_to_outer_bindings/001/main.sol" => {
            Case {
                checks: vec![Check::EntryLastReturnVariable {
                    name: "seed".into(),
                }],
                frontend_clean: Some(true),
                loader: Loader::StdCurrent,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/runtime_fields_in_callable_aggregates_do_not_become_comptime/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/runtime_fields_in_callable_aggregates_do_not_become_comptime/002/main.sol" => {
            Case {
                checks: vec![],
                diagnostics: vec![DiagnosticCheck {
                    kind: DiagnosticKind::ComptimeEvaluationFailed,
                    context: None,
                    context_contains: None,
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/scoped_declarations_do_not_residualize_shadowed_function_aggregates/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/scoped_declarations_do_not_residualize_shadowed_function_aggregates/002/main.sol" => {
            Case {
                checks: vec![Check::NoTupleBinding {
                    name: "bundle".into(),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/superclass_methods_require_an_actual_parent_implementation/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/superclass_methods_require_an_actual_parent_implementation/002/main.sol" => {
            Case {
                checks: vec![],
                diagnostics: vec![DiagnosticCheck {
                    kind: DiagnosticKind::MissingEvidence,
                    context: None,
                    context_contains: None,
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/trait_instance_calls_keep_comptime_qualified_type_arguments/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/trait_instance_calls_keep_comptime_qualified_type_arguments/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/user_concat_lit_retains_its_word_signature_and_behavior/001/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("7".into()),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/trait_method_symbols_include_method_local_type_arguments/001/main.sol" => {
            Case {
                checks: vec![
                    Check::FunctionCount {
                        selector: Selector {
                            instance_method: true,
                            ..Selector::default()
                        },
                        count: Some(2),
                        min: None,
                        max: None,
                    },
                    Check::UniqueFunctionNames,
                    Check::DistinctReturnTypes {
                        selector: Selector {
                            instance_method: true,
                            ..Selector::default()
                        },
                    },
                ],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/trait_methods_allow_unused_method_type_parameters/001/main.sol" => {
            Case {
                checks: vec![Check::FunctionCount {
                    selector: Selector {
                        instance_method: true,
                        ..Selector::default()
                    },
                    count: None,
                    min: Some(1),
                    max: None,
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/unused_superclass_selections_do_not_reject_child_methods/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/unused_superclass_selections_do_not_reject_child_methods/002/main.sol" => {
            Case {
                checks: vec![Check::MainReturnNumber {
                    value: Some("99".into()),
                }],
                loader: Loader::Bare,
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/unused_unavailable_superclasses_have_finite_recursive_dictionary_identity/001/main.sol"
        | "crates/specialize/tests/fixtures/audit/unused_unavailable_superclasses_have_finite_recursive_dictionary_identity/003/main.sol"
        | "crates/specialize/tests/fixtures/audit/unused_unavailable_superclasses_have_finite_recursive_dictionary_identity/005/main.sol"
        | "crates/specialize/tests/fixtures/audit/unused_unavailable_superclasses_have_finite_recursive_dictionary_identity/007/main.sol" => {
            Case {
                checks: vec![Check::FunctionCount {
                    selector: Selector::default(),
                    count: None,
                    min: None,
                    max: Some(9),
                }],
                forbidden_diagnostics: vec![DiagnosticKind::InstantiationDepthExceeded],
                loader: Loader::Bare,
                max_depth: Some(16),
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/audit/unused_unavailable_superclasses_have_finite_recursive_dictionary_identity/002/main.sol"
        | "crates/specialize/tests/fixtures/audit/unused_unavailable_superclasses_have_finite_recursive_dictionary_identity/004/main.sol"
        | "crates/specialize/tests/fixtures/audit/unused_unavailable_superclasses_have_finite_recursive_dictionary_identity/006/main.sol"
        | "crates/specialize/tests/fixtures/audit/unused_unavailable_superclasses_have_finite_recursive_dictionary_identity/008/main.sol" => {
            Case {
                checks: vec![Check::FunctionCount {
                    selector: Selector::default(),
                    count: None,
                    min: None,
                    max: Some(9),
                }],
                diagnostics: vec![DiagnosticCheck {
                    kind: DiagnosticKind::MissingEvidence,
                    context: None,
                    context_contains: None,
                }],
                forbidden_diagnostics: vec![DiagnosticKind::InstantiationDepthExceeded],
                loader: Loader::Bare,
                max_depth: Some(16),
                ..Case::default()
            }
        }
        "crates/specialize/tests/fixtures/function_value_evidence/main.sol" => Case {
            checks: vec![
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("a_localA_".into()),
                        ..Selector::default()
                    },
                    value: "3".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("a_qualifiedA_".into()),
                        ..Selector::default()
                    },
                    value: "3".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("a_argumentA_".into()),
                        ..Selector::default()
                    },
                    value: "3".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("a_nestedA_".into()),
                        ..Selector::default()
                    },
                    value: "3".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("a_genericA_".into()),
                        ..Selector::default()
                    },
                    value: "3".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("b_localB_".into()),
                        ..Selector::default()
                    },
                    value: "4".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("b_qualifiedB_".into()),
                        ..Selector::default()
                    },
                    value: "4".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("b_argumentB_".into()),
                        ..Selector::default()
                    },
                    value: "4".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("b_nestedB_".into()),
                        ..Selector::default()
                    },
                    value: "4".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("b_genericB_".into()),
                        ..Selector::default()
                    },
                    value: "4".into(),
                },
            ],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/module_local_function_evidence/main.sol" => Case {
            checks: vec![Check::MainReturnNumber {
                value: Some("3".into()),
            }],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/module_local_function_evidence/multiple.sol" => Case {
            checks: vec![Check::MainReturnNumber {
                value: Some("33".into()),
            }],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/module_local_function_evidence/ordered_a.sol" => Case {
            checks: vec![Check::MainReturnNumber {
                value: Some("4".into()),
            }],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/module_local_function_evidence/ordered_b.sol" => Case {
            checks: vec![Check::MainReturnNumber {
                value: Some("5".into()),
            }],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/module_local_function_evidence/values.sol" => Case {
            checks: vec![
                Check::FunctionReturns {
                    follow_calls: false,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("a_valueA_".into()),
                        ..Selector::default()
                    },
                    value: "1".into(),
                },
                Check::FunctionReturns {
                    follow_calls: false,
                    all_paths: false,
                    selector: Selector {
                        prefix: Some("b_valueB_".into()),
                        ..Selector::default()
                    },
                    value: "2".into(),
                },
            ],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/operator_evidence/main.sol"
        | "crates/specialize/tests/fixtures/operator_evidence/reversed.sol" => Case {
            checks: vec![Check::MainReturnNumber {
                value: Some("21".into()),
            }],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/superclass_dictionaries/main.sol"
        | "crates/specialize/tests/fixtures/superclass_dictionaries/reversed.sol" => Case {
            checks: vec![
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: true,
                    selector: Selector {
                        prefix: Some("a_valueA_".into()),
                        ..Selector::default()
                    },
                    value: "1".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: true,
                    selector: Selector {
                        prefix: Some("a_methodValueA_".into()),
                        ..Selector::default()
                    },
                    value: "1".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: true,
                    selector: Selector {
                        prefix: Some("b_valueB_".into()),
                        ..Selector::default()
                    },
                    value: "2".into(),
                },
                Check::FunctionReturns {
                    follow_calls: true,
                    all_paths: true,
                    selector: Selector {
                        prefix: Some("b_methodValueB_".into()),
                        ..Selector::default()
                    },
                    value: "2".into(),
                },
            ],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        "crates/specialize/tests/fixtures/trait_method_instantiations/main.sol"
        | "crates/specialize/tests/fixtures/trait_method_instantiations/reversed.sol" => Case {
            checks: vec![
                Check::FunctionCount {
                    selector: Selector {
                        method: Some("choose".into()),
                        ..Selector::default()
                    },
                    count: Some(2),
                    min: None,
                    max: None,
                },
                Check::FunctionCount {
                    selector: Selector {
                        method: Some("local".into()),
                        ..Selector::default()
                    },
                    count: Some(2),
                    min: None,
                    max: None,
                },
                Check::EqualParameterTypes {
                    selector: Selector {
                        method: Some("local".into()),
                        ..Selector::default()
                    },
                },
                Check::UniqueFunctionNames,
                Check::NoClosureDispatch,
            ],
            frontend_clean: Some(true),
            loader: Loader::Fixture,
            ..Case::default()
        },
        _ => panic!("missing expectations for source fixture {path}"),
    }
}
