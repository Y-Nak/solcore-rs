use super::*;
use dir_test::{Fixture, dir_test};

#[derive(Clone, Copy, Debug)]
enum RootLayout {
    Separate,
    MainOnly,
    Shared,
}

fn assert_diagnostics_contain(actual: &[Diagnostic], code: &str, message: &str) {
    assert!(
        actual
            .iter()
            .any(|d| d.code.as_deref() == Some(code) && d.message.contains(message)),
        "missing {code} containing {message:?}: {actual:#?}",
    );
}

fn param_shape(param: &solcore_hir_ty::AbiParam) -> String {
    let mut shape = param.ty.to_string();
    if !param.components.is_empty() {
        shape.push('(');
        shape.push_str(
            &param
                .components
                .iter()
                .map(param_shape)
                .collect::<Vec<_>>()
                .join(","),
        );
        shape.push(')');
    }
    shape
}

fn assert_param_shapes(actual: &[solcore_hir_ty::AbiParam], expected: &[&str]) {
    assert_eq!(actual.iter().map(param_shape).collect::<Vec<_>>(), expected);
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/contract_audit", glob: "**/main.sol")]
fn contract_semantics(fixture: Fixture<&str>) {
    let case_dir = PathBuf::from(fixture.path()).parent().unwrap().to_owned();
    let name = case_dir.file_name().unwrap().to_str().unwrap();
    let layouts: &[RootLayout] = match name {
        "canonical_abi_metadata_under_main_root_rejects_user_instances"
        | "canonical_abi_providers_under_main_root_remain_trusted"
        | "canonical_dispatch_under_main_root_preserves_manual_abi_diagnostics" => {
            &[RootLayout::MainOnly, RootLayout::Shared]
        }
        _ => &[RootLayout::Separate],
    };
    for &roots in layouts {
        let (mut db, key) = match roots {
            RootLayout::Separate => {
                let (mut db, key) = db_with_main(fixture.content());
                insert_real_std_modules(&mut db);
                (db, key)
            }
            RootLayout::MainOnly => db_with_overlapping_std(fixture.content(), false),
            RootLayout::Shared => db_with_overlapping_std(fixture.content(), true),
        };
        let project_root = match roots {
            RootLayout::Separate => "/main",
            RootLayout::MainOnly | RootLayout::Shared => "/std",
        };
        for entry in std::fs::read_dir(&case_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "sol")
                && path.file_name().unwrap() != "main.sol"
            {
                let name = path.file_stem().unwrap().to_str().unwrap();
                insert_module_source(
                    &mut db,
                    ModuleKey {
                        library: LibraryId::Main,
                        logical_path: vec![name.to_owned()],
                    },
                    &format!("{project_root}/{name}.sol"),
                    &std::fs::read_to_string(&path).unwrap(),
                );
            }
        }
        // Keep specialized semantic checks in Rust, next to the assertions they
        // exercise. Source files alone define the set of directory test cases.
        match name {
            "canonical_abi_metadata_under_main_root_rejects_user_instances" => {
                let diagnostics = diagnostics_for_module(&db, &key);
                assert_eq!(diagnostics.len(), 2, "{roots:?}: {diagnostics:#?}");
                assert_diagnostics_contain(&diagnostics, "SC0231", "manual `ABIElementAttribs`");
                assert_diagnostics_contain(&diagnostics, "SC0231", "manual `ABITypeString`");
            }
            "canonical_abi_providers_under_main_root_remain_trusted"
            | "unselected_manual_element_metadata_remains_available_to_internal_code" => {
                let diagnostics = diagnostics_for_module(&db, &key);
                assert!(diagnostics.is_empty(), "{roots:?}: {diagnostics:#?}");
            }
            "canonical_dispatch_under_main_root_preserves_manual_abi_diagnostics" => {
                let module_id = module_id_from_key(&db, &key);
                let diagnostics = nameres::reachable_diagnostics(&db, module_id);
                assert!(diagnostics.is_empty(), "{roots:?}: {diagnostics:#?}");
                let diagnostics = diagnostics_for_module(&db, &key);
                assert_eq!(diagnostics.len(), 4, "{roots:?}: {diagnostics:#?}");
                for class in ["ABIAttribs", "ABIEncode", "ABIDecode", "SigString"] {
                    assert_diagnostics_contain(
                        &diagnostics,
                        "SC0231",
                        &format!("manual `{class}`"),
                    );
                }
            }
            "external_array_abi_rejects_selected_manual_element_layout_nested"
            | "external_array_abi_rejects_selected_manual_element_layout_unit"
            | "external_array_abi_rejects_selected_manual_element_layout_word" => {
                let diagnostics = diagnostics_for_module(&db, &key);
                assert!(
                    diagnostics
                        .iter()
                        .all(|d| d.code.as_deref() == Some("SC0231")),
                    "{diagnostics:#?}"
                );
                assert_diagnostics_contain(&diagnostics, "SC0231", "manual `ABIElementAttribs`");
            }
            "external_array_abi_rejects_selected_manual_element_signature" => {
                assert_diagnostics_contain(
                    &diagnostics_for_module(&db, &key),
                    "SC0231",
                    "manual `ABITypeString`",
                );
            }
            "external_nested_array_abi_rejects_imported_manual_element_layout" => {
                assert_diagnostics_contain(
                    &diagnostics_for_module(&db, &key),
                    "SC0231",
                    "manual `ABIElementAttribs`",
                );
            }
            "raw_abi_fallback_requires_supported_primitive" => {
                let diagnostics = diagnostics_for_module(&db, &key);
                assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
                assert!(
                    diagnostics
                        .iter()
                        .all(|d| d.code.as_deref() == Some("SC0207")),
                    "{diagnostics:#?}"
                );
                assert_diagnostics_contain(&diagnostics, "SC0207", "Custom: ABIEncode");
                assert_diagnostics_contain(
                    &diagnostics,
                    "SC0207",
                    "ABIDecoder<Custom, MemoryWordReader>: ABIDecode<Custom>",
                );
            }
            "explicit_unit_product_members_are_rejected_at_public_boundaries_array_tuple"
            | "explicit_unit_product_members_are_rejected_at_public_boundaries_constructor"
            | "explicit_unit_product_members_are_rejected_at_public_boundaries_nested_array_tuple"
            | "explicit_unit_product_members_are_rejected_at_public_boundaries_parameter"
            | "explicit_unit_product_members_are_rejected_at_public_boundaries_return_tuple"
            | "explicit_unit_product_members_are_rejected_at_public_boundaries_tuple"
            | "unit_array_outputs_remain_input_only" => {
                let module = parse_file_to_hir(&db, db.module_files[&key]).module(&db);
                let contract = contract_named(&db, module, "C");
                let surface = contract_dispatch_surface(&db, module, contract);
                if name == "unit_array_outputs_remain_input_only" {
                    assert_diagnostics_contain(&surface.diagnostics, "SC0231", "input-only");
                    assert_eq!(
                        surface
                            .diagnostics
                            .iter()
                            .filter(|d| d.code.as_deref() == Some("SC0231")
                                && d.message.contains("input-only"))
                            .count(),
                        2,
                        "{:?}",
                        surface.diagnostics
                    );
                } else {
                    assert_diagnostics_contain(&surface.diagnostics, "SC0231", "explicit unit");
                }
                assert!(contract_abi_json(&db, module, contract).is_err());
            }
            "calldata_tuple_array_uses_canonical_signature_and_json_components" => {
                let module = parse_file_to_hir(&db, db.module_files[&key]).module(&db);
                let contract = contract_named(&db, module, "Tuples");
                let surface = contract_dispatch_surface(&db, module, contract);
                assert_eq!(surface.methods.len(), 1);
                let method = &surface.methods[0];
                assert_eq!(method.signature, "first((uint256,address)[])");
                assert_eq!(method.selector.to_hex(), "0x01de5ccf");
                assert_param_shapes(&method.inputs, &["tuple[](uint256,address)"]);
                let abi = contract_abi_json(&db, module, contract).unwrap();
                assert!(abi.contains("\"type\": \"tuple[]\""), "{abi}");
                assert!(abi.contains("\"components\": ["), "{abi}");
            }
            "unit_array_abi_keeps_empty_tuple_value_context" => {
                let module = parse_file_to_hir(&db, db.module_files[&key]).module(&db);
                let contract = contract_named(&db, module, "UnitArrays");
                let surface = contract_dispatch_surface(&db, module, contract);
                assert!(surface.diagnostics.is_empty(), "{:?}", surface.diagnostics);
                assert_eq!(surface.methods.len(), 4);
                let expected = [
                    ("units(()[])", vec!["tuple[]"]),
                    ("nested(()[][])", vec!["tuple[][]"]),
                    ("empty()", vec![]),
                    (
                        "wrapped((()[],uint256)[])",
                        vec!["tuple[](tuple[],uint256)"],
                    ),
                ];
                for (method, (signature, inputs)) in surface.methods.iter().zip(expected) {
                    assert_eq!(method.signature, signature);
                    assert_param_shapes(&method.inputs, &inputs);
                }
                assert_eq!(surface.methods[0].selector.to_hex(), "0xfc59547a");
                assert!(surface.methods[2].outputs.is_empty());
                let DispatchConstructor::Explicit { inputs, .. } = &surface.constructor else {
                    panic!("expected explicit constructor");
                };
                assert_param_shapes(inputs, &["uint256"]);
                let abi = contract_abi_json(&db, module, contract).unwrap();
                for text in ["\"type\": \"tuple[]\"", "\"type\": \"tuple[][]\""] {
                    assert!(abi.contains(text), "{text}: {abi}");
                }
                for text in ["\"type\": \"\"", "\"type\": \"[]\""] {
                    assert!(!abi.contains(text), "{text}: {abi}");
                }
                assert_eq!(abi.matches("\"components\": []").count(), 3, "{abi}");
            }
            other => panic!("missing contract assertions for fixture: {other}"),
        }
    }
}
