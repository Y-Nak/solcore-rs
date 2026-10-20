//! Rename support over the wasm-clean LSP core.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
};

use lsp_types::{Location, Position, PrepareRenameResponse, Range, TextEdit, Url, WorkspaceEdit};

use crate::{
    LineIndexExt,
    references::{
        collect_reference_locations, preserves_pattern_resolutions, reference_target_at,
        target_accepts_identifier, target_supports_text_rename,
    },
    state::WorldState,
};

/// Computes the rename range for the user symbol at a source position.
pub fn handle_prepare_rename(
    world: &WorldState,
    uri: &Url,
    position: Position,
) -> Option<PrepareRenameResponse> {
    let target = reference_target_at(world, uri, position)?;
    if !target_supports_text_rename(world, &target) {
        return None;
    }
    let line_index = world.line_index(uri)?;
    let offset = line_index.position_to_byte(position)?;
    let locations = editable_reference_locations(world, &target)?;

    locations
        .into_iter()
        .filter(|location| location.uri == *uri)
        .find_map(|location| {
            location_contains_offset(line_index, &location, offset).then_some(location.range)
        })
        .map(PrepareRenameResponse::Range)
}

/// Computes a workspace edit that renames the user symbol at a source position.
pub fn handle_rename(
    world: &WorldState,
    uri: &Url,
    position: Position,
    new_name: &str,
) -> Option<WorkspaceEdit> {
    let target = reference_target_at(world, uri, position)?;
    if !target_accepts_identifier(world, &target, new_name) {
        return None;
    }

    if !target_supports_text_rename(world, &target) {
        return None;
    }
    let locations = editable_reference_locations(world, &target)?;
    let changes = text_edits_by_uri(locations, new_name)?;
    if !preserves_pattern_resolutions(world, &changes) {
        return None;
    }

    Some(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

fn editable_reference_locations(
    world: &WorldState,
    target: &crate::references::ReferenceTarget,
) -> Option<Vec<Location>> {
    let locations = collect_reference_locations(world, target, true);
    (!locations.is_empty()
        && locations
            .iter()
            .all(|location| world.line_index(&location.uri).is_some()))
    .then_some(locations)
}

fn text_edits_by_uri(
    locations: Vec<Location>,
    new_name: &str,
) -> Option<HashMap<Url, Vec<TextEdit>>> {
    let mut locations_by_uri = BTreeMap::<String, (Url, Vec<Range>)>::new();
    for location in locations {
        locations_by_uri
            .entry(location.uri.as_str().to_owned())
            .or_insert_with(|| (location.uri.clone(), Vec::new()))
            .1
            .push(location.range);
    }

    let mut changes = HashMap::new();
    for (_, (uri, mut ranges)) in locations_by_uri {
        ranges.sort_by(compare_ranges);
        ranges.dedup_by(|left, right| left.start == right.start && left.end == right.end);
        if ranges_overlap(&ranges) {
            return None;
        }

        let edits = ranges
            .into_iter()
            .map(|range| TextEdit {
                range,
                new_text: new_name.to_owned(),
            })
            .collect();
        changes.insert(uri, edits);
    }

    Some(changes)
}

fn location_contains_offset(line_index: &LineIndexExt, location: &Location, offset: u32) -> bool {
    let Some(start) = line_index.position_to_byte(location.range.start) else {
        return false;
    };
    let Some(end) = line_index.position_to_byte(location.range.end) else {
        return false;
    };

    start <= offset && offset < end
}

fn ranges_overlap(ranges: &[Range]) -> bool {
    ranges
        .windows(2)
        .any(|pair| compare_positions(&pair[1].start, &pair[0].end).is_lt())
}

fn compare_ranges(left: &Range, right: &Range) -> Ordering {
    compare_positions(&left.start, &right.start)
        .then_with(|| compare_positions(&left.end, &right.end))
}

fn compare_positions(left: &Position, right: &Position) -> Ordering {
    left.line
        .cmp(&right.line)
        .then_with(|| left.character.cmp(&right.character))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with_main(source: &str) -> (WorldState, Url) {
        let mut world = WorldState::new();
        let uri = Url::parse("file:///main/main.sol").expect("uri");
        assert!(world.open_document(uri.clone(), source.to_owned()));
        (world, uri)
    }

    fn world_with_main_and_math(main: &str, math: &str) -> (WorldState, Url, Url) {
        let mut world = WorldState::new();
        let main_uri = Url::parse("file:///main/main.sol").expect("main uri");
        let math_uri = Url::parse("file:///main/math.sol").expect("math uri");
        assert!(world.open_document(main_uri.clone(), main.to_owned()));
        assert!(world.open_document(math_uri.clone(), math.to_owned()));
        (world, main_uri, math_uri)
    }

    #[test]
    fn renaming_hidden_import_updates_provider_and_hiding_occurrence() {
        let main = "import * from math hiding {hidden};\nimport {hidden as kept} from math;\nfunction main() returns (word) { return kept(); }\n";
        let math = "export {hidden};\nfunction hidden() returns (word) { return 7; }\n";
        for from_hiding in [false, true] {
            let (mut world, main_uri, math_uri) = world_with_main_and_math(main, math);
            let (uri, source) = if from_hiding {
                (&main_uri, main)
            } else {
                (&math_uri, math)
            };
            let position = world
                .line_index(uri)
                .unwrap()
                .byte_to_position(source.find("hidden").unwrap() as u32);
            let edit =
                handle_rename(&world, uri, position, "renamed").expect("rename hidden symbol");
            apply_workspace_edit(&mut world, edit);
            assert!(
                world
                    .document_text(&main_uri)
                    .unwrap()
                    .contains("hiding {renamed}")
            );
            assert!(
                world
                    .document_text(&main_uri)
                    .unwrap()
                    .contains("{renamed as kept}")
            );
            assert!(!world.document_text(&math_uri).unwrap().contains("hidden"));
            assert_no_error_diagnostics(&world, &[main_uri, math_uri]);
        }
    }

    #[test]
    fn renaming_hidden_selected_import_updates_every_provider_reference() {
        let math = "export {hidden};\nfunction hidden() returns (word) { return 7; }\n";
        for selector in ["hidden", "hidden as kept"] {
            let main = format!(
                "import {{{selector}}} from math hiding {{hidden}};\nfunction main() returns (word) {{ return 1; }}\n"
            );
            for origin in ["provider", "selector", "hiding"] {
                let (mut world, main_uri, math_uri) = world_with_main_and_math(&main, math);
                assert_no_error_diagnostics(&world, &[main_uri.clone(), math_uri.clone()]);
                let (uri, offset) = match origin {
                    "provider" => (&math_uri, math.find("hidden").unwrap()),
                    "selector" => (&main_uri, main.find("hidden").unwrap()),
                    _ => (&main_uri, main.rfind("hidden").unwrap()),
                };
                let position = world
                    .line_index(uri)
                    .unwrap()
                    .byte_to_position(offset as u32);
                assert!(handle_prepare_rename(&world, uri, position).is_some());
                let edit = handle_rename(&world, uri, position, "renamed").expect(origin);
                apply_workspace_edit(&mut world, edit);
                let expected = main.replace("hidden", "renamed");
                assert_eq!(world.document_text(&main_uri), Some(expected.as_str()));
                assert_eq!(
                    world.document_text(&math_uri),
                    Some(math.replace("hidden", "renamed").as_str())
                );
                assert_no_error_diagnostics(&world, &[main_uri, math_uri]);
            }
        }
    }

    #[test]
    fn renaming_hidden_type_import_preserves_its_alias() {
        let math = "enum Choice { Some(word), None }\nexport {Choice(Some)};\n";
        let main = "import {Choice as Hidden} from math hiding {Choice};\nfunction main() returns (word) { return 1; }\n";
        for from_selector in [false, true] {
            let (mut world, main_uri, math_uri) = world_with_main_and_math(main, math);
            assert_no_error_diagnostics(&world, &[main_uri.clone(), math_uri.clone()]);
            let (uri, source) = if from_selector {
                (&main_uri, main)
            } else {
                (&math_uri, math)
            };
            let position = world
                .line_index(uri)
                .unwrap()
                .byte_to_position(source.find("Choice").unwrap() as u32);
            let edit = handle_rename(&world, uri, position, "Option").expect("rename hidden type");
            apply_workspace_edit(&mut world, edit);
            assert_eq!(
                world.document_text(&main_uri),
                Some(main.replace("Choice", "Option").as_str())
            );
            assert_eq!(
                world.document_text(&math_uri),
                Some(math.replace("Choice", "Option").as_str())
            );
            assert_no_error_diagnostics(&world, &[main_uri, math_uri]);
        }
    }

    #[test]
    fn renaming_trait_method_updates_all_impls_with_the_same_trait_identity() {
        let math = "export {Echo};\ntrait Echo<t> { function echo(x: t) returns (word); }\n";
        let main = r#"import {Echo} from math;
impl Echo<word> { function echo(x: word) returns (word) { return x; } }
impl Echo<bool> { function echo(x: bool) returns (word) { return 0; } }
trait Other<t> { function echo(x: t) returns (word); }
impl Other<word> { function echo(x: word) returns (word) { return x; } }
function main() returns (word) { let x: word = 7; return Echo.echo(x); }
"#;
        for from_impl in [false, true] {
            let (mut world, main_uri, math_uri) = world_with_main_and_math(main, math);
            let (uri, source) = if from_impl {
                (&main_uri, main)
            } else {
                (&math_uri, math)
            };
            let position = world
                .line_index(uri)
                .unwrap()
                .byte_to_position(source.find("echo").unwrap() as u32);
            let edit = handle_rename(&world, uri, position, "apply").expect("rename trait method");
            apply_workspace_edit(&mut world, edit);
            let renamed = world.document_text(&main_uri).unwrap();
            assert_eq!(renamed.matches("function apply").count(), 2, "{renamed}");
            assert_eq!(renamed.matches("function echo").count(), 2, "{renamed}");
            assert!(renamed.contains("Echo.apply(x)"));
            assert!(
                world
                    .document_text(&math_uri)
                    .unwrap()
                    .contains("function apply")
            );
            assert_no_error_diagnostics(&world, &[main_uri, math_uri]);
        }
    }

    fn apply_workspace_edit(world: &mut WorldState, edit: WorkspaceEdit) {
        for (uri, mut edits) in edit.changes.expect("text edits") {
            let mut text = world.document_text(&uri).expect("document text").to_owned();
            let index = LineIndexExt::new(&text);
            edits.sort_by(|a, b| compare_ranges(&b.range, &a.range));
            for edit in edits {
                let start = index.position_to_byte(edit.range.start).unwrap() as usize;
                let end = index.position_to_byte(edit.range.end).unwrap() as usize;
                text.replace_range(start..end, &edit.new_text);
            }
            assert!(world.change_document(&uri, text));
        }
    }

    fn assert_no_error_diagnostics(world: &WorldState, uris: &[Url]) {
        for uri in uris {
            let diagnostics = crate::diagnostics::compute_diagnostics(world, uri);
            assert!(
                diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.severity
                        != Some(lsp_types::DiagnosticSeverity::ERROR)),
                "{uri}: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn rename_source_bindings_includes_assembly_reads_and_assignments() {
        for source in [
            "function main(input: word) returns (word) { let value: word = input; assembly { value := add(value, 1) } return value; }",
            "function main(value: word) returns (word) { assembly { value := add(value, 1) } return value; }",
            "function main(input: word) returns (word) { match (input) { case value { assembly { value := add(value, 1) } return value; } } }",
            "function main(input: word) returns (word) { let f = lam (value: word) -> word { assembly { value := add(value, 1) } return value; }; return f(input); }",
            "function main(input: word) returns (word) { let f = lam (x: word) -> word { let value = x; assembly { value := add(value, 1) } return value; }; return f(input); }",
            "function main(input: word) returns (word) { let value = input; let f = lam (x: word) -> word { assembly { pop(value) } return value; }; return f(input); }",
            "function main(value: word) returns (word) { let other: word; assembly { value, other := pair(value) function pair(input) -> a, b { a := input b := input } } return value; }",
            "function main(input: word) returns (word) { if (true) { let value = input; } assembly { value := add(value, 1) } return value; }",
            "import * from std; import * from std.dispatch; contract C { function update(value: word) returns (word) { assembly { value := add(value, 1) } return value; } }",
        ] {
            // Every occurrence is also a valid rename origin, including Yul
            // assignment targets and reads, parameters and pattern bindings.
            for (offset, _) in source.match_indices("value") {
                let (mut world, uri) = world_with_main(source);
                assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
                let position = world
                    .line_index(&uri)
                    .unwrap()
                    .byte_to_position(offset as u32);
                assert!(
                    handle_prepare_rename(&world, &uri, position).is_some(),
                    "{source}"
                );
                let edit = handle_rename(&world, &uri, position, "renamed").expect(source);
                apply_workspace_edit(&mut world, edit);
                assert_eq!(
                    world.document_text(&uri),
                    Some(source.replace("value", "renamed").as_str())
                );
                assert_no_error_diagnostics(&world, &[uri]);
            }
        }
    }

    #[test]
    fn rename_assembly_references_preserves_yul_scopes_and_namespaces() {
        let source = r#"function main(value: word) returns (word) {
    assembly {
        pop(value)
        { let value := value pop(value) }
        if value { let value := value pop(value) }
        for { let value := value } value { value := sub(value, 1) } {
            let temp := value
            pop(temp)
        }
        for { let i := value } i { value := add(value, 1) i := sub(i, 1) } {
            let value := i
            pop(value)
        }
        switch value
        case 0 { let value := value pop(value) }
        default { value := add(value, 1) }
        function value(value) -> result {
            function nested(value) -> result { result := value }
            result := nested(value)
        }
        function other() -> value { value := 7 }
        value := value(value)
    }
    assembly { value := add(value, 1) }
    return value;
}"#;
        let expected = r#"function main(renamed: word) returns (word) {
    assembly {
        pop(renamed)
        { let value := renamed pop(value) }
        if renamed { let value := renamed pop(value) }
        for { let value := renamed } value { value := sub(value, 1) } {
            let temp := value
            pop(temp)
        }
        for { let i := renamed } i { renamed := add(renamed, 1) i := sub(i, 1) } {
            let value := i
            pop(value)
        }
        switch renamed
        case 0 { let value := renamed pop(value) }
        default { renamed := add(renamed, 1) }
        function value(value) -> result {
            function nested(value) -> result { result := value }
            result := nested(value)
        }
        function other() -> value { value := 7 }
        renamed := value(renamed)
    }
    assembly { renamed := add(renamed, 1) }
    return renamed;
}"#;
        for offset in [
            source.find("value:").unwrap(),
            source.find("pop(value)").unwrap() + 4,
        ] {
            let (mut world, uri) = world_with_main(source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(offset as u32);
            let edit = handle_rename(&world, &uri, position, "renamed").expect("source rename");
            apply_workspace_edit(&mut world, edit);
            assert_eq!(world.document_text(&uri), Some(expected));
            assert_no_error_diagnostics(&world, &[uri]);
        }
    }

    #[test]
    fn rename_assembly_reference_rejects_capture_by_a_yul_local() {
        for assembly in [
            "let renamed := 1 value := add(value, renamed)",
            "{ let renamed := 1 pop(value) }",
            "for { let renamed := 1 } value { renamed := 0 } { pop(value) }",
        ] {
            let source = format!(
                "function main(value: word) returns (word) {{ assembly {{ {assembly} }} return value; }}"
            );
            let (world, uri) = world_with_main(&source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(source.find("value").unwrap() as u32);
            assert!(handle_rename(&world, &uri, position, "renamed").is_none());
            assert!(handle_rename(&world, &uri, position, "safeName").is_some());
        }
    }

    #[test]
    fn rename_assembly_reference_ignores_locals_in_separate_scopes() {
        let source = "function main(value: word) returns (word) { assembly { { let renamed := 1 pop(renamed) } function f(renamed) -> result { result := renamed } value := f(value) } return value; }";
        let (mut world, uri) = world_with_main(source);
        assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
        let position = world
            .line_index(&uri)
            .unwrap()
            .byte_to_position(source.find("value").unwrap() as u32);
        let edit = handle_rename(&world, &uri, position, "renamed").expect("non-capturing rename");
        apply_workspace_edit(&mut world, edit);
        assert_eq!(
            world.document_text(&uri),
            Some(source.replace("value", "renamed").as_str())
        );
        assert_no_error_diagnostics(&world, &[uri]);
    }

    #[test]
    fn rename_assembly_reference_keeps_source_shadowing_bindings_separate() {
        let source = "function main(value: word) returns (word) { assembly { pop(value) } { let value: word = 7; assembly { pop(value) } } return value; }";
        let expected = "function main(renamed: word) returns (word) { assembly { pop(renamed) } { let value: word = 7; assembly { pop(value) } } return renamed; }";
        let (mut world, uri) = world_with_main(source);
        assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
        let position = world
            .line_index(&uri)
            .unwrap()
            .byte_to_position(source.find("pop(value)").unwrap() as u32 + 4);
        let edit = handle_rename(&world, &uri, position, "renamed").expect("source-scoped rename");
        apply_workspace_edit(&mut world, edit);
        assert_eq!(world.document_text(&uri), Some(expected));
        assert_no_error_diagnostics(&world, &[uri]);
    }

    #[test]
    fn rename_rejects_source_bindings_that_capture_edited_references() {
        for source in [
            "function main(value: word) returns (word) { { let renamed: word = 7; assembly { value := add(value, renamed) } } return value; }",
            "function main(value: word) returns (word) { { let renamed: word = 7; value = primAddWord(value, renamed); } return value; }",
            "function main(value: word, flag: bool) returns (word) { if (flag) { let renamed: word = 7; } assembly { value := add(value, renamed) } return value; }",
            "function main(value: word) returns (word) { for (; false; let renamed: word = 7) { assembly { value := renamed } } return value; }",
            "function main(value: word) returns (word) { match (value) { case (renamed) { assembly { value := add(value, renamed) } } } return value; }",
            "function main(value: word) returns (word) { let f = lam (renamed: word) -> word { return primAddWord(value, renamed); }; return f(7); }",
            "function main(value: word) returns (word) { let f = lam (renamed: word) -> word { let out: word; assembly { out := add(value, renamed) } return out; }; return f(7); }",
        ] {
            let (world, uri) = world_with_main(source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            for (offset, _) in source.match_indices("value") {
                let position = world
                    .line_index(&uri)
                    .unwrap()
                    .byte_to_position(offset as u32);
                assert!(
                    handle_rename(&world, &uri, position, "renamed").is_none(),
                    "{source}"
                );
                assert!(
                    handle_rename(&world, &uri, position, "safeName").is_some(),
                    "{source}"
                );
            }
        }
    }

    #[test]
    fn rename_rejects_capture_of_unedited_source_and_assembly_references() {
        for source in [
            "function main(renamed: word) returns (word) { let value: word = 7; return primAddWord(value, renamed); }",
            "function main(renamed: word) returns (word) { let value: word = 7; assembly { renamed := add(renamed, value) } return renamed; }",
            "function main(renamed: word) returns (word) { let value: word = 7; return renamed; }",
            "function main(renamed: word) returns (word) { let value: word = 7; let f = lam () -> word { return renamed; }; return f(); }",
            "function main(renamed: word) returns (word) { match (renamed) { case (value) { return renamed; } } }",
            "function main(renamed: word) returns (word) { let f = lam (value: word) -> word { return renamed; }; return f(7); }",
            "function renamed() returns (word) { return 42; } function main() returns (word) { let value: word = 7; return renamed(); }",
            "import * from std; import * from std.dispatch; contract C { renamed: word; function read() returns (word) { let value: word = 7; return renamed; } }",
        ] {
            let (world, uri) = world_with_main(source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(source.find("value").unwrap() as u32);
            assert!(
                handle_rename(&world, &uri, position, "renamed").is_none(),
                "{source}"
            );
            assert!(
                handle_rename(&world, &uri, position, "safeName").is_some(),
                "{source}"
            );
        }
    }

    #[test]
    fn rename_cannot_capture_an_imported_module_qualifier() {
        let main = "import * as renamed from math; function main() returns (word) { let value: word = 7; return renamed.read(); }";
        let math = "export {read}; function read() returns (word) { return 42; }";
        let (world, uri, math_uri) = world_with_main_and_math(main, math);
        assert_no_error_diagnostics(&world, &[uri.clone(), math_uri]);
        let position = world
            .line_index(&uri)
            .unwrap()
            .byte_to_position(main.find("value").unwrap() as u32);
        assert!(handle_rename(&world, &uri, position, "renamed").is_none());
        assert!(handle_rename(&world, &uri, position, "safeName").is_some());
    }

    #[test]
    fn rename_items_and_fields_rejects_capture_by_source_bindings() {
        for source in [
            "function target() returns (word) { return 1; } function main() returns (word) { let renamed = lam () -> word { return 2; }; return target(); }",
            "function target() returns (word) { return 1; } function main() returns (word) { let run = lam (renamed: word) -> word { return target(); }; return run(2); }",
            "import * from std; import * from std.dispatch; contract C { target: word; function read() returns (word) { let renamed: word = 2; return target; } }",
        ] {
            let (world, uri) = world_with_main(source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            for (offset, _) in source.match_indices("target") {
                let position = world
                    .line_index(&uri)
                    .unwrap()
                    .byte_to_position(offset as u32);
                assert!(
                    handle_rename(&world, &uri, position, "renamed").is_none(),
                    "{source}"
                );
                assert!(
                    handle_rename(&world, &uri, position, "safeName").is_some(),
                    "{source}"
                );
            }
        }
    }

    #[test]
    fn rename_imported_items_and_aliases_rejects_capture_at_consumer() {
        let math = "export {target}; function target() returns (word) { return 1; }";
        for import in [
            "import {target} from math;",
            "import {target as alias} from math;",
            "import * as M from math;",
        ] {
            let (name, expression) = if import.contains("alias") {
                ("alias", "alias()")
            } else if import.contains("as M") {
                ("M", "M.target()")
            } else {
                ("target", "target()")
            };
            let main = format!(
                "{import} function main() returns (word) {{ let renamed = lam () -> word {{ return 2; }}; return {expression}; }}"
            );
            let (world, uri, math_uri) = world_with_main_and_math(&main, math);
            assert_no_error_diagnostics(&world, &[uri.clone(), math_uri.clone()]);
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(main.find(name).unwrap() as u32);
            assert!(
                handle_rename(&world, &uri, position, "renamed").is_none(),
                "{main}"
            );
            assert!(
                handle_rename(&world, &uri, position, "safeName").is_some(),
                "{main}"
            );
            if name == "alias" {
                // Renaming the provider leaves the alias spelling unchanged.
                let position = world
                    .line_index(&math_uri)
                    .unwrap()
                    .byte_to_position(math.find("target").unwrap() as u32);
                assert!(handle_rename(&world, &math_uri, position, "renamed").is_some());
            }
        }
    }

    #[test]
    fn rename_function_preserves_initializer_and_disjoint_scope_lookup() {
        for source in [
            "function target() returns (word) { return 1; } function main() returns (word) { let renamed = target(); return renamed; }",
            "function target() returns (word) { return 1; } function main() returns (word) { { let renamed: word = 2; } return target(); }",
            "function target() returns (word) { return 1; } function main() returns (word) { let out = target(); let renamed: word = 2; return out; }",
        ] {
            let (mut world, uri) = world_with_main(source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(source.find("target").unwrap() as u32);
            let edit = handle_rename(&world, &uri, position, "renamed").expect(source);
            apply_workspace_edit(&mut world, edit);
            assert_no_error_diagnostics(&world, &[uri]);
        }
    }

    #[test]
    fn rename_pattern_binder_preserves_its_role_in_every_pattern_form() {
        for (declarations, input, pattern) in [
            ("enum Choice { Choice, Other }", "input: Choice", "(value)"),
            (
                "enum Choice { Choice, Other }",
                "input: (Choice, word)",
                "((value, other))",
            ),
            (
                "enum Choice { Choice, Other } enum Box { Box(Choice) }",
                "input: Box",
                "Box(value)",
            ),
        ] {
            let source = format!(
                "{declarations} function main({input}) returns (word) {{ match (input) {{ case {pattern} {{ return 7; }} default {{ return 9; }} }} }}"
            );
            let (mut world, uri) = world_with_main(&source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(source.find("value").unwrap() as u32);
            for constructor in ["Choice", "Other"] {
                assert!(
                    handle_rename(&world, &uri, position, constructor).is_none(),
                    "{source}"
                );
            }
            let edit =
                handle_rename(&world, &uri, position, "renamed").expect("safe pattern rename");
            apply_workspace_edit(&mut world, edit);
            assert_no_error_diagnostics(&world, &[uri]);
        }
    }

    #[test]
    fn rename_pattern_binder_observes_imported_and_contract_constructor_visibility() {
        let math = "enum Choice { Choice, Other } export {Choice(Choice, Other)};";
        for main in [
            "import {Choice} from math; function main(input: Choice) returns (word) { match (input) { case (value) { return 7; } } }",
            "import * from std; import * from std.dispatch; contract C { enum Choice { Choice, Other } function main() returns (word) { match (Choice()) { case (value) { return 7; } } } }",
        ] {
            let (world, uri, math_uri) = world_with_main_and_math(main, math);
            assert_no_error_diagnostics(&world, &[uri.clone(), math_uri]);
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(main.find("value").unwrap() as u32);
            assert!(
                handle_rename(&world, &uri, position, "Choice").is_none(),
                "{main}"
            );
            assert!(
                handle_rename(&world, &uri, position, "Other").is_none(),
                "{main}"
            );
            assert!(
                handle_rename(&world, &uri, position, "renamed").is_some(),
                "{main}"
            );
        }
    }

    #[test]
    fn rename_constructor_cannot_capture_an_unedited_pattern_binder() {
        let math = "enum Choice { Pick, Other } export {Choice(Pick, Other)};";
        for import in ["import {Choice} from math;", "import * from math;"] {
            let main = format!(
                "{import} function main(input: Choice) returns (word) {{ match (input) {{ case (value) {{ return 7; }} }} }}"
            );
            let (world, uri, math_uri) = world_with_main_and_math(&main, math);
            assert_no_error_diagnostics(&world, &[uri, math_uri.clone()]);
            let position = world
                .line_index(&math_uri)
                .unwrap()
                .byte_to_position(math.find("Pick").unwrap() as u32);
            assert!(handle_rename(&world, &math_uri, position, "value").is_none());
            assert!(handle_rename(&world, &math_uri, position, "Renamed").is_some());
        }
    }

    #[test]
    fn rename_type_preserves_unqualified_constructor_pattern_roles() {
        let source = "enum Choice { Choice, Other } function main(input: Choice) returns (word) { match (input) { case (Choice) { return 7; } default { return 9; } } }";
        let (world, uri) = world_with_main(source);
        assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
        let position = world
            .line_index(&uri)
            .unwrap()
            .byte_to_position(source.find("Choice").unwrap() as u32);
        assert!(handle_rename(&world, &uri, position, "Renamed").is_none());
        assert_eq!(world.document_text(&uri), Some(source));
    }

    #[test]
    fn rename_constructor_respects_hidden_and_contract_local_visibility() {
        for (main, math) in [
            (
                "import {Choice} from math; function main(input: Choice) returns (word) { match (input) { case (value) { return 7; } } }",
                "enum Choice { Pick, Other } export {Choice};",
            ),
            (
                "import * from std; import * from std.dispatch; contract C { enum Choice { Pick, Other } } function main(input: word) returns (word) { match (input) { case (value) { return 7; } } }",
                "",
            ),
        ] {
            let (mut world, uri, math_uri) = world_with_main_and_math(main, math);
            assert_no_error_diagnostics(&world, &[uri.clone(), math_uri.clone()]);
            let pattern_position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(main.find("value").unwrap() as u32);
            assert!(
                handle_rename(&world, &uri, pattern_position, "Pick").is_some(),
                "hidden constructors must not restrict a binder's name"
            );
            let (source, target_uri) = if math.is_empty() {
                (main, &uri)
            } else {
                (math, &math_uri)
            };
            let position = world
                .line_index(target_uri)
                .unwrap()
                .byte_to_position(source.find("Pick").unwrap() as u32);
            let edit = handle_rename(&world, target_uri, position, "value")
                .expect("constructor is not visible at the pattern");
            apply_workspace_edit(&mut world, edit);
            assert_no_error_diagnostics(&world, &[uri, math_uri]);
        }
    }

    #[test]
    fn rename_pattern_validation_maps_unicode_and_length_changing_edits() {
        let source = "// 🦀\nenum Choice { Choice, Other } function fλ(input: Choice) returns (word) { match (input) { case (value) { return 7; } } } function main() returns (word) { return fλ(Choice()); }";
        let (mut world, uri) = world_with_main(source);
        assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
        for (old_name, new_name) in [("fλ", "renamed漢"), ("value", "bindingλ")] {
            let before = world.document_text(&uri).unwrap().to_owned();
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(before.find(old_name).unwrap() as u32);
            let edit =
                handle_rename(&world, &uri, position, new_name).expect("mapped pattern identities");
            assert_eq!(
                world.document_text(&uri),
                Some(before.as_str()),
                "validation must not edit the live document"
            );
            apply_workspace_edit(&mut world, edit);
            assert_eq!(
                world.document_text(&uri),
                Some(before.replace(old_name, new_name).as_str())
            );
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
        }
    }

    #[test]
    fn rename_preserves_initializer_and_disjoint_scope_lookup() {
        for source in [
            "function main(value: word) returns (word) { let renamed: word = value; return renamed; }",
            "function main(value: word) returns (word) { { let renamed: word = value; assembly { pop(renamed) } } return value; }",
            "function main(value: word) returns (word) { { let renamed: word = 7; assembly { pop(renamed) } } return value; }",
            "function main(value: word) returns (word) { let f = lam () -> word { let renamed: word = value; return renamed; }; return f(); }",
            "function main(value: word) returns (word) { assembly { pop(value) } let renamed: word = 7; return renamed; }",
            "function main(value: word) returns (word) { for (; false; let renamed: word = value) { assembly { pop(renamed) } } return renamed; }",
            "function main(value: word) returns (word) { match (7) { case (renamed) { assembly { pop(renamed) } } } return value; }",
        ] {
            let (mut world, uri) = world_with_main(source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(source.find("value").unwrap() as u32);
            let edit = handle_rename(&world, &uri, position, "renamed").expect(source);
            apply_workspace_edit(&mut world, edit);
            assert_eq!(
                world.document_text(&uri),
                Some(source.replace("value", "renamed").as_str())
            );
            assert_no_error_diagnostics(&world, &[uri]);
        }
    }

    #[test]
    fn rename_respects_runtime_parameter_contexts() {
        for source in [
            "function id(value: word) returns (word) { return value; }",
            "import * from std; import * from std.dispatch; contract C { function id(value: word) returns (word) { return value; } }",
            "trait Echo<t> { function echo(x: t) returns (t); } impl Echo<word> { function echo(value: word) returns (word) { return value; } }",
            "function main() returns (word) { let f = lam (value: word) -> word { return value; }; return f(1); }",
            "function main() returns (word) { let f = lam (value) { return value; }; return f(1); }",
            "function main() returns (word) { let f = lam (outer: word) { let g = lam (value) { return value; }; return g(outer); }; return f(1); }",
        ] {
            let (world, uri) = world_with_main(source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            for offset in [
                source.find("value").unwrap(),
                source.rfind("value").unwrap(),
            ] {
                let position = world
                    .line_index(&uri)
                    .unwrap()
                    .byte_to_position(offset as u32);
                assert!(
                    handle_prepare_rename(&world, &uri, position).is_some(),
                    "{source}"
                );
                assert!(
                    handle_rename(&world, &uri, position, "comptime").is_none(),
                    "{source}"
                );
                assert!(
                    handle_rename(&world, &uri, position, "renamed").is_some(),
                    "{source}"
                );
            }
        }
    }

    #[test]
    fn rename_preserves_legal_contextual_names() {
        for (source, new_name) in [
            (
                "function id(comptime value: word) returns (word) { return value; }",
                "comptime",
            ),
            (
                "function main() returns (word) { let f = lam (comptime value: word) -> word { return value; }; return f(1); }",
                "comptime",
            ),
            (
                "function main() returns (word) { let value: word = 1; return value; }",
                "comptime",
            ),
            (
                "function value() returns (word) { return 1; } function main() returns (word) { return value(); }",
                "comptime",
            ),
            (
                "function id(value: word) returns (word) { return value; }",
                "while",
            ),
            (
                "function id(value: word) returns (word) { return value; }",
                "mapping",
            ),
            (
                "function id(value: word) returns (word) { return value; }",
                "derive",
            ),
        ] {
            let (mut world, uri) = world_with_main(source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(source.find("value").unwrap() as u32);
            let edit = handle_rename(&world, &uri, position, new_name)
                .unwrap_or_else(|| panic!("{source} -> {new_name}"));
            apply_workspace_edit(&mut world, edit);
            assert_no_error_diagnostics(&world, &[uri]);
        }
    }

    #[test]
    fn rename_rejects_contextual_type_syntax_at_unqualified_references() {
        for (source, new_name) in [
            (
                "type Original = word; function id(value: Original) returns (Original) { return value; }",
                "mapping",
            ),
            (
                "enum Original { Value } function id(value: Original) returns (Original) { return value; }",
                "mapping",
            ),
            (
                "function id<Original>(value: Original) returns (Original) { return value; }",
                "mapping",
            ),
            ("type Identity(Original) = (Original, word);", "mapping"),
            (
                "trait Identity<Original> { function id(value: Original) returns (Original); }",
                "mapping",
            ),
            (
                "type Original = word; function main(value: word) returns (word) { let local: Original = value; return local; }",
                "mapping",
            ),
            (
                "type Original = word; function main(value: word) returns (word) { let f = lam (item: Original) -> Original { return item; }; return f(value); }",
                "mapping",
            ),
            (
                "import * from std; import * from std.dispatch; contract C { type Original = word; function id(value: Original) returns (Original) { return value; } }",
                "mapping",
            ),
            (
                "type Original(a) = a; function id(value: Original<word>) returns (Original<word>) { return value; }",
                "comptime",
            ),
            (
                "enum Original<a> { Value(a) } function id(value: Original<word>) returns (Original<word>) { return value; }",
                "comptime",
            ),
        ] {
            let (world, uri) = world_with_main(source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            for offset in [
                source.find("Original").unwrap(),
                source.rfind("Original").unwrap(),
            ] {
                let position = world
                    .line_index(&uri)
                    .unwrap()
                    .byte_to_position(offset as u32);
                assert!(
                    handle_prepare_rename(&world, &uri, position).is_some(),
                    "{source}"
                );
                assert!(
                    handle_rename(&world, &uri, position, new_name).is_none(),
                    "{source} -> {new_name}"
                );
            }
        }
    }

    #[test]
    fn rename_validates_imported_type_alias_uses_and_unqualified_consumers() {
        let math = "type Box(a) = a; export {Box};";
        for import in [
            "import {Box as Original} from math;",
            "import {Box} from math;",
        ] {
            let name = if import.contains(" as ") {
                "Original"
            } else {
                "Box"
            };
            let main = format!(
                "{import} function id(value: {name}<word>) returns (word) {{ return value; }}"
            );
            let (world, main_uri, math_uri) = world_with_main_and_math(&main, math);
            assert_no_error_diagnostics(&world, &[main_uri.clone(), math_uri.clone()]);
            for offset in [main.find(name).unwrap(), main.rfind(name).unwrap()] {
                let position = world
                    .line_index(&main_uri)
                    .unwrap()
                    .byte_to_position(offset as u32);
                assert!(handle_prepare_rename(&world, &main_uri, position).is_some());
                for new_name in ["mapping", "comptime"] {
                    assert!(
                        handle_rename(&world, &main_uri, position, new_name).is_none(),
                        "{main} -> {new_name}"
                    );
                }
            }
            if name == "Box" {
                let position = world
                    .line_index(&math_uri)
                    .unwrap()
                    .byte_to_position(math.find(name).unwrap() as u32);
                assert!(handle_rename(&world, &math_uri, position, "mapping").is_none());
            }
        }
    }

    #[test]
    fn rename_preserves_legal_contextual_type_and_value_names() {
        for (source, new_name) in [
            (
                "type Original = word; function id(value: Original) returns (Original) { return value; }",
                "comptime",
            ),
            (
                "type Original(a, b) = (a, b); function id(value: Original<word, bool>) returns (Original<word, bool>) { return value; }",
                "comptime",
            ),
            (
                "function Original() returns (word) { return 7; } function main() returns (word) { return Original(); }",
                "mapping",
            ),
            ("type Original = word;", "mapping"),
            ("trait Empty<Original> {}", "mapping"),
            (
                "function id<Original>(value: Original) returns (Original) { return value; }",
                "comptime",
            ),
            (
                "trait Original<t> { function id(value: t) returns (t); } impl Original<word> { function id(value: word) returns (word) { return value; } }",
                "mapping",
            ),
        ] {
            let (mut world, uri) = world_with_main(source);
            assert_no_error_diagnostics(&world, std::slice::from_ref(&uri));
            let position = world
                .line_index(&uri)
                .unwrap()
                .byte_to_position(source.find("Original").unwrap() as u32);
            let edit = handle_rename(&world, &uri, position, new_name)
                .unwrap_or_else(|| panic!("{source} -> {new_name}"));
            apply_workspace_edit(&mut world, edit);
            assert_no_error_diagnostics(&world, &[uri]);
        }

        let math = "type Original(a) = a; export {Original};";
        let main = "import * as M from math; function id(value: M.Original<word>) returns (M.Original<word>) { return value; }";
        for new_name in ["mapping", "comptime"] {
            for from_use in [false, true] {
                let (mut world, main_uri, math_uri) = world_with_main_and_math(main, math);
                assert_no_error_diagnostics(&world, &[main_uri.clone(), math_uri.clone()]);
                let (uri, source) = if from_use {
                    (&main_uri, main)
                } else {
                    (&math_uri, math)
                };
                let position = world
                    .line_index(uri)
                    .unwrap()
                    .byte_to_position(source.find("Original").unwrap() as u32);
                let edit =
                    handle_rename(&world, uri, position, new_name).expect("qualified type rename");
                apply_workspace_edit(&mut world, edit);
                assert!(
                    world
                        .document_text(&main_uri)
                        .unwrap()
                        .contains(&format!("M.{new_name}<word>"))
                );
                assert_no_error_diagnostics(&world, &[main_uri, math_uri]);
            }
        }

        let main = "import {Original as Alias} from math; function id(value: Alias<word>) returns (word) { return value; }";
        for new_name in ["mapping", "comptime"] {
            let (mut world, main_uri, math_uri) = world_with_main_and_math(main, math);
            let position = world
                .line_index(&math_uri)
                .unwrap()
                .byte_to_position(math.find("Original").unwrap() as u32);
            let edit = handle_rename(&world, &math_uri, position, new_name)
                .expect("provider rename preserves a consumer alias");
            apply_workspace_edit(&mut world, edit);
            assert!(
                world
                    .document_text(&main_uri)
                    .unwrap()
                    .contains("Alias<word>")
            );
            assert_no_error_diagnostics(&world, &[main_uri, math_uri]);
        }
    }

    #[test]
    fn renaming_parameter_edits_declaration_and_uses() {
        let source = "function id(x: word) returns (word) {\n  let y = x;\n  return x;\n}\n";
        let (world, uri) = world_with_main(source);
        let line_index = world.line_index(&uri).expect("line index");
        let declaration = source.find("x: word").expect("declaration") as u32;
        let first_use = (source.find("let y = x").expect("first use") + "let y = ".len()) as u32;
        let second_use = (source.find("return x").expect("second use") + "return ".len()) as u32;
        let position = line_index.byte_to_position(first_use);

        let edit = handle_rename(&world, &uri, position, "renamed").expect("rename edit");

        assert_eq!(edit.document_changes, None);
        assert_eq!(edit.change_annotations, None);
        let changes = edit.changes.expect("changes");
        let edits = changes.get(&uri).expect("current file edits");
        assert_eq!(edits.len(), 3);
        assert!(edits.iter().all(|edit| edit.new_text == "renamed"));
        assert_eq!(
            edits.iter().map(|edit| edit.range).collect::<Vec<_>>(),
            vec![
                line_index.range(declaration, declaration + 1),
                line_index.range(first_use, first_use + 1),
                line_index.range(second_use, second_use + 1),
            ]
        );
    }

    #[test]
    fn prepare_rename_returns_user_symbol_range_but_not_builtin_or_keyword() {
        let source = "function id(x: word) returns (word) {\n  return x;\n}\n";
        let (world, uri) = world_with_main(source);
        let line_index = world.line_index(&uri).expect("line index");
        let use_offset = (source.find("return x").expect("use") + "return ".len()) as u32;

        let prepare = handle_prepare_rename(&world, &uri, line_index.byte_to_position(use_offset))
            .expect("prepare rename");
        match prepare {
            PrepareRenameResponse::Range(range) => {
                assert_eq!(range, line_index.range(use_offset, use_offset + 1));
            }
            other => panic!("expected range prepare response, got {other:?}"),
        }

        let builtin = source.find("word").expect("builtin") as u32;
        assert_eq!(
            handle_prepare_rename(&world, &uri, line_index.byte_to_position(builtin)),
            None
        );
        let keyword = source.find("return").expect("keyword") as u32;
        assert_eq!(
            handle_prepare_rename(&world, &uri, line_index.byte_to_position(keyword)),
            None
        );
    }

    #[test]
    fn rename_rejects_invalid_new_name() {
        let source = "function id(x: word) returns (word) {\n  return x;\n}\n";
        let (world, uri) = world_with_main(source);
        let line_index = world.line_index(&uri).expect("line index");
        let use_offset = (source.find("return x").expect("use") + "return ".len()) as u32;
        let position = line_index.byte_to_position(use_offset);

        assert!(handle_rename(&world, &uri, position, "1bad").is_none());
        assert!(handle_rename(&world, &uri, position, "").is_none());
        assert!(handle_rename(&world, &uri, position, "return").is_none());
        assert!(handle_rename(&world, &uri, position, "bad-name").is_none());
        assert!(handle_rename(&world, &uri, position, "_bad").is_none());
        assert!(handle_rename(&world, &uri, position, "λvalue").is_some());
    }

    #[test]
    fn renaming_exported_function_edits_import_and_export_names() {
        let main =
            "import {double} from math;\nfunction main() returns (word) { return double(21); }\n";
        let math =
            "function double(x: word) returns (word) { return x + x; }\nexport { double };\n";
        let (world, main_uri, math_uri) = world_with_main_and_math(main, math);
        let main_index = world.line_index(&main_uri).expect("main line index");
        let math_index = world.line_index(&math_uri).expect("math line index");
        let import = main.find("double").expect("import") as u32;
        let call = main.rfind("double").expect("call") as u32;
        let declaration = math.find("double").expect("declaration") as u32;
        let export = math.rfind("double").expect("export") as u32;

        let edit = handle_rename(
            &world,
            &main_uri,
            main_index.byte_to_position(call),
            "twice",
        )
        .expect("rename edit");
        let changes = edit.changes.expect("changes");

        let main_edits = changes.get(&main_uri).expect("main edits");
        assert!(main_edits.iter().all(|edit| edit.new_text == "twice"));
        assert_eq!(
            main_edits.iter().map(|edit| edit.range).collect::<Vec<_>>(),
            vec![
                main_index.range(import, import + "double".len() as u32),
                main_index.range(call, call + "double".len() as u32),
            ]
        );

        let math_edits = changes.get(&math_uri).expect("math edits");
        assert!(math_edits.iter().all(|edit| edit.new_text == "twice"));
        assert_eq!(
            math_edits.iter().map(|edit| edit.range).collect::<Vec<_>>(),
            vec![
                math_index.range(declaration, declaration + "double".len() as u32),
                math_index.range(export, export + "double".len() as u32),
            ]
        );
    }

    #[test]
    fn embedded_std_symbol_is_not_offered_for_rename() {
        let source = "import {addWord} from std;\nfunction main() returns (word) { return addWord(1, 2); }\n";
        let (world, uri) = world_with_main(source);
        let line_index = world.line_index(&uri).expect("line index");
        let call = source.rfind("addWord").expect("call") as u32;
        let position = line_index.byte_to_position(call);

        assert_eq!(handle_prepare_rename(&world, &uri, position), None);
        assert_eq!(handle_rename(&world, &uri, position, "sumWords"), None);
    }

    #[test]
    fn renaming_exported_function_from_defining_module_edits_importer() {
        let mut world = WorldState::new();
        let main_uri = Url::parse("file:///main/main.sol").expect("main uri");
        let math_uri = Url::parse("file:///main/math.sol").expect("math uri");
        let main =
            "import {double} from math;\nfunction main() returns (word) { return double(21); }\n";
        let math = "function double(x: word) returns (word) { return x + x; }\nfunction local() returns (word) { return double(2); }\nexport { double };\n";
        assert!(world.open_document(math_uri.clone(), math.to_owned()));
        assert!(world.open_document(main_uri.clone(), main.to_owned()));
        let main_index = world.line_index(&main_uri).expect("main line index");
        let math_index = world.line_index(&math_uri).expect("math line index");
        let import = main.find("double").expect("import") as u32;
        let call = main.rfind("double").expect("call") as u32;
        let declaration = math.find("double").expect("declaration") as u32;
        let local_call = math.find("double(2)").expect("local call") as u32;
        let export = math.rfind("double").expect("export") as u32;

        let edit = handle_rename(
            &world,
            &math_uri,
            math_index.byte_to_position(local_call),
            "twice",
        )
        .expect("rename edit");
        let changes = edit.changes.expect("changes");

        let main_edits = changes.get(&main_uri).expect("main edits");
        assert!(main_edits.iter().all(|edit| edit.new_text == "twice"));
        assert_eq!(
            main_edits.iter().map(|edit| edit.range).collect::<Vec<_>>(),
            vec![
                main_index.range(import, import + "double".len() as u32),
                main_index.range(call, call + "double".len() as u32),
            ]
        );

        let math_edits = changes.get(&math_uri).expect("math edits");
        assert!(math_edits.iter().all(|edit| edit.new_text == "twice"));
        assert_eq!(
            math_edits.iter().map(|edit| edit.range).collect::<Vec<_>>(),
            vec![
                math_index.range(declaration, declaration + "double".len() as u32),
                math_index.range(local_call, local_call + "double".len() as u32),
                math_index.range(export, export + "double".len() as u32),
            ]
        );
    }

    #[test]
    fn renaming_selected_import_alias_only_edits_local_alias_uses() {
        let main = "import {double as twice} from math;\nfunction main() returns (word) { return twice(21); }\n";
        let math = "function double(x: word) returns (word) { return x; }\nexport { double };\n";
        let (world, main_uri, math_uri) = world_with_main_and_math(main, math);
        let index = world.line_index(&main_uri).expect("main index");
        let alias = main.find("twice").expect("alias declaration") as u32;
        let use_offset = main.rfind("twice").expect("alias use") as u32;

        let edit = handle_rename(
            &world,
            &main_uri,
            index.byte_to_position(use_offset),
            "applyTwice",
        )
        .expect("alias rename");
        let changes = edit.changes.expect("changes");
        assert!(!changes.contains_key(&math_uri));
        assert_eq!(
            changes
                .get(&main_uri)
                .expect("main edits")
                .iter()
                .map(|edit| edit.range)
                .collect::<Vec<_>>(),
            vec![
                index.range(alias, alias + "twice".len() as u32),
                index.range(use_offset, use_offset + "twice".len() as u32),
            ]
        );
    }

    #[test]
    fn renaming_explicit_module_alias_edits_alias_and_qualifiers() {
        let main =
            "import * as M from math;\nfunction main() returns (word) { return M.value(); }\n";
        let math = "function value() returns (word) { return 1; }\nexport { value };\n";
        let (world, main_uri, _) = world_with_main_and_math(main, math);
        let index = world.line_index(&main_uri).expect("main index");
        let alias = main.find("M from").expect("alias declaration") as u32;
        let use_offset = main.rfind("M.value").expect("alias use") as u32;

        let edit = handle_rename(
            &world,
            &main_uri,
            index.byte_to_position(use_offset),
            "Math",
        )
        .expect("module alias rename");
        let edits = edit
            .changes
            .expect("changes")
            .remove(&main_uri)
            .expect("main edits");
        assert_eq!(
            edits.iter().map(|edit| edit.range).collect::<Vec<_>>(),
            vec![
                index.range(alias, alias + 1),
                index.range(use_offset, use_offset + 1),
            ]
        );
    }

    #[test]
    fn renaming_module_alias_updates_type_and_pattern_qualifiers() {
        let main = "import * as M from math;\nfunction unwrap(token: M.Token) returns (word) {\n  match (token) {\n  case M.Token.Ok(value) { return value; }\ncase M.Token.Err(value) { return value; }}\n}\n";
        let model = "enum Token {Ok(word) , Err(word)}\nexport { Token(Ok, Err) };\n";
        let (world, main_uri, _) = world_with_main_and_math(main, model);
        let index = world.line_index(&main_uri).expect("main index");
        let declaration = main.find("M from").expect("alias declaration") as u32;
        let type_qualifier = main.find("M.Token").expect("type qualifier") as u32;
        let ok_qualifier = main.find("M.Token.Ok").expect("Ok qualifier") as u32;
        let err_qualifier = main.find("M.Token.Err").expect("Err qualifier") as u32;

        let edit = handle_rename(
            &world,
            &main_uri,
            index.byte_to_position(type_qualifier),
            "Model",
        )
        .expect("module alias rename");
        let edits = edit
            .changes
            .expect("changes")
            .remove(&main_uri)
            .expect("main edits");
        assert_eq!(
            edits.iter().map(|edit| edit.range).collect::<Vec<_>>(),
            vec![
                index.range(declaration, declaration + 1),
                index.range(type_qualifier, type_qualifier + 1),
                index.range(ok_qualifier, ok_qualifier + 1),
                index.range(err_qualifier, err_qualifier + 1),
            ]
        );
    }

    #[test]
    fn exported_selected_alias_is_not_offered_an_incomplete_text_rename() {
        let main = "import {double as twice} from math;\nexport { twice };\nfunction main() returns (word) { return twice(21); }\n";
        let math = "function double(x: word) returns (word) { return x; }\nexport { double };\n";
        let (world, main_uri, _) = world_with_main_and_math(main, math);
        let index = world.line_index(&main_uri).expect("main index");
        let use_offset = main.rfind("twice").expect("alias use") as u32;
        let position = index.byte_to_position(use_offset);

        assert_eq!(handle_prepare_rename(&world, &main_uri, position), None);
        assert_eq!(handle_rename(&world, &main_uri, position, "thrice"), None);
    }

    #[test]
    fn renaming_exported_module_alias_updates_downstream_qualifiers() {
        let mut world = WorldState::new();
        let util_uri = Url::parse("file:///main/util.sol").expect("util uri");
        let facade_uri = Url::parse("file:///main/facade.sol").expect("facade uri");
        let main_uri = Url::parse("file:///main/main.sol").expect("main uri");
        let consumer_uri = Url::parse("file:///main/consumer.sol").expect("consumer uri");
        let util = "function value() returns (word) { return 1; }\nexport { value };\n";
        let facade = "export util as Tools;\n";
        let main =
            "import facade;\nfunction main() returns (word) { return facade.Tools.value(); }\n";
        let consumer =
            "import facade;\nfunction consume() returns (word) { return facade.Tools.value(); }\n";
        assert!(world.open_document(util_uri, util.to_owned()));
        assert!(world.open_document(facade_uri.clone(), facade.to_owned()));
        assert!(world.open_document(main_uri.clone(), main.to_owned()));
        assert!(world.open_document(consumer_uri.clone(), consumer.to_owned()));
        let facade_index = world.line_index(&facade_uri).expect("facade index");
        let main_index = world.line_index(&main_uri).expect("main index");
        let consumer_index = world.line_index(&consumer_uri).expect("consumer index");
        let declaration = facade.find("Tools").expect("export alias") as u32;
        let qualifier = main.find("Tools").expect("qualified alias") as u32;
        let consumer_qualifier = consumer.find("Tools").expect("consumer qualifier") as u32;

        let edit = handle_rename(
            &world,
            &main_uri,
            main_index.byte_to_position(qualifier),
            "Helpers",
        )
        .expect("exported module alias rename");
        let changes = edit.changes.expect("changes");
        assert_eq!(
            changes[&facade_uri]
                .iter()
                .map(|edit| edit.range)
                .collect::<Vec<_>>(),
            vec![facade_index.range(declaration, declaration + 5)]
        );
        assert_eq!(
            changes[&main_uri]
                .iter()
                .map(|edit| edit.range)
                .collect::<Vec<_>>(),
            vec![main_index.range(qualifier, qualifier + 5)]
        );
        assert_eq!(
            changes[&consumer_uri]
                .iter()
                .map(|edit| edit.range)
                .collect::<Vec<_>>(),
            vec![consumer_index.range(consumer_qualifier, consumer_qualifier + 5)]
        );
        assert!(
            changes[&consumer_uri]
                .iter()
                .all(|edit| edit.new_text == "Helpers")
        );
    }

    #[test]
    fn source_definition_rename_is_rejected_across_exported_selected_alias() {
        let mut world = WorldState::new();
        let math_uri = Url::parse("file:///main/math.sol").expect("math uri");
        let bridge_uri = Url::parse("file:///main/bridge.sol").expect("bridge uri");
        let main_uri = Url::parse("file:///main/main.sol").expect("main uri");
        let math = "function double(x: word) returns (word) { return x; }\nexport { double };\n";
        let bridge = "import {double as twice} from math;\nexport { twice };\n";
        let main =
            "import {twice} from bridge;\nfunction main() returns (word) { return twice(1); }\n";
        assert!(world.open_document(math_uri.clone(), math.to_owned()));
        assert!(world.open_document(bridge_uri, bridge.to_owned()));
        assert!(world.open_document(main_uri, main.to_owned()));
        let index = world.line_index(&math_uri).expect("math index");
        let declaration = math.find("double").expect("source declaration") as u32;
        let position = index.byte_to_position(declaration);

        assert_eq!(handle_prepare_rename(&world, &math_uri, position), None);
        assert_eq!(handle_rename(&world, &math_uri, position, "timesTwo"), None);
    }

    #[test]
    fn multi_root_rename_never_edits_same_spelling_in_another_root() {
        let base = std::env::temp_dir().join("solcore-lsp-rename-multi-root");
        let left_path = base.join("left");
        let right_path = base.join("right");
        let left_root = Url::from_directory_path(&left_path).expect("left root");
        let right_root = Url::from_directory_path(&right_path).expect("right root");
        let left_main = Url::from_file_path(left_path.join("main.sol")).expect("left main");
        let left_math = Url::from_file_path(left_path.join("math.sol")).expect("left math");
        let right_main = Url::from_file_path(right_path.join("main.sol")).expect("right main");
        let right_math = Url::from_file_path(right_path.join("math.sol")).expect("right math");
        let left_source =
            "import {value} from lib.math;\nfunction left() returns (word) { return value(); }\n";
        let right_source =
            "import {value} from lib.math;\nfunction right() returns (word) { return value(); }\n";
        let left_library = "function value() returns (word) { return 1; }\nexport { value };\n";
        let right_library = "function value() returns (word) { return 2; }\nexport { value };\n";
        let mut world = WorldState::new();
        world.load_workspace_roots([
            (
                left_root,
                vec![
                    (left_main.clone(), left_source.to_owned()),
                    (left_math.clone(), left_library.to_owned()),
                ],
            ),
            (
                right_root,
                vec![
                    (right_main.clone(), right_source.to_owned()),
                    (right_math.clone(), right_library.to_owned()),
                ],
            ),
        ]);
        assert!(world.open_document(left_main.clone(), left_source.to_owned()));
        let index = world.line_index(&left_main).expect("left index");
        let use_offset = left_source.rfind("value").expect("left use") as u32;

        let edit = handle_rename(
            &world,
            &left_main,
            index.byte_to_position(use_offset),
            "renamed",
        )
        .expect("left rename");
        let changes = edit.changes.expect("changes");
        assert_eq!(changes.len(), 2);
        assert!(changes.contains_key(&left_main));
        assert!(changes.contains_key(&left_math));
        assert!(!changes.contains_key(&right_main));
        assert!(!changes.contains_key(&right_math));
    }

    #[test]
    fn renaming_exported_module_alias_updates_unaliased_reexport_chain() {
        let mut world = WorldState::new();
        let util_uri = Url::parse("file:///main/util.sol").expect("util uri");
        let facade_uri = Url::parse("file:///main/facade.sol").expect("facade uri");
        let bridge_uri = Url::parse("file:///main/bridge.sol").expect("bridge uri");
        let main_uri = Url::parse("file:///main/main.sol").expect("main uri");
        let util = "function value() returns (word) { return 1; }\nexport { value };\n";
        let facade = "export util as Tools;\n";
        let bridge = "export facade;\n";
        let main = "import bridge;\nfunction main() returns (word) { return bridge.facade.Tools.value(); }\n";
        assert!(world.open_document(util_uri, util.to_owned()));
        assert!(world.open_document(facade_uri.clone(), facade.to_owned()));
        assert!(world.open_document(bridge_uri.clone(), bridge.to_owned()));
        assert!(world.open_document(main_uri.clone(), main.to_owned()));
        let facade_index = world.line_index(&facade_uri).expect("facade index");
        let main_index = world.line_index(&main_uri).expect("main index");
        let declaration = facade.find("Tools").expect("alias declaration") as u32;
        let use_offset = main.find("Tools").expect("downstream use") as u32;

        let edit = handle_rename(
            &world,
            &main_uri,
            main_index.byte_to_position(use_offset),
            "Helpers",
        )
        .expect("transitive module alias rename");
        let changes = edit.changes.expect("changes");
        assert_eq!(
            changes[&facade_uri]
                .iter()
                .map(|edit| edit.range)
                .collect::<Vec<_>>(),
            vec![facade_index.range(declaration, declaration + 5)]
        );
        assert!(!changes.contains_key(&bridge_uri));
        assert_eq!(
            changes[&main_uri]
                .iter()
                .map(|edit| edit.range)
                .collect::<Vec<_>>(),
            vec![main_index.range(use_offset, use_offset + 5)]
        );
    }

    #[test]
    fn default_module_reexport_without_alias_is_not_text_renameable() {
        let mut world = WorldState::new();
        let util_uri = Url::parse("file:///main/util.sol").expect("util uri");
        let facade_uri = Url::parse("file:///main/facade.sol").expect("facade uri");
        let main_uri = Url::parse("file:///main/main.sol").expect("main uri");
        let util = "function value() returns (word) { return 1; }\nexport { value };\n";
        let facade = "export util;\n";
        let main =
            "import facade;\nfunction main() returns (word) { return facade.util.value(); }\n";
        assert!(world.open_document(util_uri, util.to_owned()));
        assert!(world.open_document(facade_uri, facade.to_owned()));
        assert!(world.open_document(main_uri.clone(), main.to_owned()));
        let index = world.line_index(&main_uri).expect("main index");
        let module = main.find("util").expect("default module alias") as u32;
        let position = index.byte_to_position(module);

        assert_eq!(handle_prepare_rename(&world, &main_uri, position), None);
        assert_eq!(handle_rename(&world, &main_uri, position, "tools"), None);
    }

    #[test]
    fn renaming_constructor_updates_import_and_export_selectors() {
        let mut world = WorldState::new();
        let main_uri = Url::parse("file:///main/main.sol").expect("main uri");
        let model_uri = Url::parse("file:///main/model.sol").expect("model uri");
        let main = "import {Token} from model;\nfunction make(x: word) returns (Token) { return Token.Ok(x); }\n";
        let model = "enum Token {Ok(word) , Err(word)}\nexport { Token(Ok, Err) };\n";
        assert!(world.open_document(main_uri.clone(), main.to_owned()));
        assert!(world.open_document(model_uri.clone(), model.to_owned()));
        let main_index = world.line_index(&main_uri).expect("main index");
        let model_index = world.line_index(&model_uri).expect("model index");
        let use_ctor = main.rfind("Ok").expect("constructor use") as u32;
        let declaration = model.find("Ok").expect("constructor declaration") as u32;
        let export_ctor = model.rfind("Ok").expect("export constructor") as u32;

        let edit = handle_rename(
            &world,
            &main_uri,
            main_index.byte_to_position(use_ctor),
            "Success",
        )
        .expect("constructor rename");
        let changes = edit.changes.expect("changes");
        assert_eq!(
            changes
                .get(&main_uri)
                .expect("main edits")
                .iter()
                .map(|edit| edit.range)
                .collect::<Vec<_>>(),
            vec![main_index.range(use_ctor, use_ctor + 2)]
        );
        assert_eq!(
            changes
                .get(&model_uri)
                .expect("model edits")
                .iter()
                .map(|edit| edit.range)
                .collect::<Vec<_>>(),
            vec![
                model_index.range(declaration, declaration + 2),
                model_index.range(export_ctor, export_ctor + 2),
            ]
        );
    }
}
