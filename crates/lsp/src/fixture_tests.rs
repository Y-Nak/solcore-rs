//! Declarative editor fixtures preserve the exact document text and UTF-16 positions.
//!
//! `.sol.txt` files are editor buffers, including incomplete syntax and deliberate
//! spacing, opened under `.sol` URIs. Each check starts with fresh documents;
//! explicit rename sequences exercise successive edits to the same workspace.

use std::{collections::BTreeMap, fs, path::Path};

use dir_test::{Fixture, dir_test};
use lsp_types::{
    CompletionItemKind, CompletionResponse, DiagnosticSeverity, GotoDefinitionResponse, Position,
    Url, WorkspaceEdit,
};
use serde::Deserialize;

use crate::{LineIndexExt, WorldState};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    documents: Vec<String>,
    checks: Vec<Check>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Check {
    Rename(Rename),
    RenameSequence {
        steps: Vec<Rename>,
    },
    Completion {
        at: Cursor,
        #[serde(default)]
        parse_clean: bool,
        contains: Vec<ExpectedCompletion>,
        #[serde(default)]
        excludes: Vec<String>,
    },
    Signature {
        at: Cursor,
        active_parameter: u32,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedCompletion {
    label: String,
    kind: CompletionItemKind,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    #[serde(default = "main_document")]
    document: String,
    needle: String,
    #[serde(default)]
    occurrence: Occurrence,
    #[serde(default)]
    offset: usize,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Occurrence {
    #[default]
    First,
    Last,
    All,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rename {
    at: Cursor,
    new_name: String,
    accepted: bool,
    #[serde(default)]
    prepare: bool,
    #[serde(default)]
    no_errors_before: bool,
    #[serde(default)]
    no_errors_after: bool,
    references: Option<usize>,
    /// Bound compiler body-resolution work independently of wall-clock speed.
    max_resolved_bodies: Option<usize>,
    definition: Option<Cursor>,
    #[serde(default)]
    expected_documents: BTreeMap<String, String>,
    #[serde(default)]
    contents: Vec<ContentExpectation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentExpectation {
    #[serde(default = "main_document")]
    document: String,
    text: String,
    count: Option<usize>,
    #[serde(default)]
    absent: bool,
}

fn main_document() -> String {
    "main.sol".to_owned()
}

fn uri(document: &str) -> Url {
    Url::parse(&format!("file:///main/{document}")).expect("fixture document URI")
}

fn open_documents(root: &Path, documents: &[String]) -> WorldState {
    let mut world = WorldState::new();
    for document in documents {
        let source =
            fs::read_to_string(root.join(format!("{document}.txt"))).expect("fixture document");
        assert!(world.open_document(uri(document), source));
    }
    world
}

fn positions(world: &WorldState, cursor: &Cursor) -> Vec<Position> {
    assert!(!cursor.needle.is_empty(), "cursor needle is empty");
    let uri = uri(&cursor.document);
    let source = world.document_text(&uri).expect("cursor document");
    let offsets: Vec<_> = source
        .match_indices(&cursor.needle)
        .map(|(offset, _)| offset)
        .collect();
    assert!(
        !offsets.is_empty(),
        "missing cursor needle {:?}",
        cursor.needle
    );
    let selected: Vec<_> = match cursor.occurrence {
        Occurrence::First => vec![offsets[0]],
        Occurrence::Last => vec![*offsets.last().unwrap()],
        Occurrence::All => offsets,
    };
    selected
        .into_iter()
        .map(|offset| {
            world
                .line_index(&uri)
                .unwrap()
                .byte_to_position((offset + cursor.offset) as u32)
        })
        .collect()
}

fn no_error_diagnostics(world: &WorldState, documents: &[String]) {
    for document in documents {
        let diagnostics = crate::compute_diagnostics(world, &uri(document));
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.severity != Some(DiagnosticSeverity::ERROR)),
            "{document}: {diagnostics:?}"
        );
    }
}

fn document_texts(world: &WorldState, documents: &[String]) -> BTreeMap<String, String> {
    documents
        .iter()
        .map(|document| {
            (
                document.clone(),
                world.document_text(&uri(document)).unwrap().to_owned(),
            )
        })
        .collect()
}

fn apply_workspace_edit(world: &mut WorldState, edit: WorkspaceEdit) {
    for (uri, mut edits) in edit.changes.expect("text edits") {
        let mut source = world
            .document_text(&uri)
            .expect("edited document")
            .to_owned();
        let index = LineIndexExt::new(&source);
        edits.sort_by_key(|edit| {
            std::cmp::Reverse((
                edit.range.start.line,
                edit.range.start.character,
                edit.range.end.line,
                edit.range.end.character,
            ))
        });
        for edit in edits {
            let start = index.position_to_byte(edit.range.start).unwrap() as usize;
            let end = index.position_to_byte(edit.range.end).unwrap() as usize;
            source.replace_range(start..end, &edit.new_text);
        }
        assert!(world.change_document(&uri, source));
    }
}

fn check_rename(
    root: &Path,
    documents: &[String],
    world: &mut WorldState,
    check: &Rename,
    position: Position,
) {
    assert!(
        check.accepted
            || (check.expected_documents.is_empty()
                && check.contents.is_empty()
                && !check.no_errors_after),
        "rejected renames cannot have post-edit expectations"
    );
    let target = uri(&check.at.document);
    if check.no_errors_before {
        no_error_diagnostics(world, documents);
    }
    if check.prepare {
        assert!(
            crate::handle_prepare_rename(world, &target, position).is_some(),
            "prepare rename"
        );
    }
    if let Some(count) = check.references {
        let references =
            crate::handle_references(world, &target, position, true).expect("references");
        assert_eq!(references.len(), count, "reference count");
    }
    if let Some(expected) = &check.definition {
        let Some(GotoDefinitionResponse::Scalar(definition)) =
            crate::handle_definition(world, &target, position)
        else {
            panic!("expected scalar definition");
        };
        let positions = positions(world, expected);
        assert_eq!(positions.len(), 1, "definition has one expected location");
        assert_eq!(definition.uri, uri(&expected.document));
        assert_eq!(definition.range.start, positions[0]);
    }
    let before = document_texts(world, documents);
    crate::references::take_resolved_body_count();
    let result = crate::handle_rename(world, &target, position, &check.new_name);
    let resolved_bodies = crate::references::take_resolved_body_count();
    if let Some(maximum) = check.max_resolved_bodies {
        assert!(
            resolved_bodies <= maximum,
            "rename resolved {resolved_bodies} bodies, expected at most {maximum}"
        );
    }
    assert_eq!(
        document_texts(world, documents),
        before,
        "rename validation must not mutate live documents"
    );
    assert_eq!(
        result.is_some(),
        check.accepted,
        "rename {:?} at {target}:{position:?}",
        check.new_name
    );
    if let Some(edit) = result {
        apply_workspace_edit(world, edit);
        for (document, expected) in &check.expected_documents {
            let expected = fs::read_to_string(root.join(expected)).expect("expected document");
            assert_eq!(
                world.document_text(&uri(document)),
                Some(expected.as_str()),
                "edited {document}"
            );
        }
        for expected in &check.contents {
            assert!(
                expected.count.is_none() || !expected.absent,
                "content expectations cannot combine count and absent"
            );
            let source = world
                .document_text(&uri(&expected.document))
                .expect("edited document");
            if let Some(count) = expected.count {
                assert_eq!(
                    source.matches(&expected.text).count(),
                    count,
                    "occurrences of {:?}",
                    expected.text
                );
            } else {
                assert_eq!(
                    source.contains(&expected.text),
                    !expected.absent,
                    "edited document {:?} contains {:?}",
                    expected.document,
                    expected.text
                );
            }
        }
        if check.no_errors_after {
            no_error_diagnostics(world, documents);
        }
    }
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/editor", glob: "**/case.json")]
fn editor_fixture(fixture: Fixture<&str>) {
    let root = Path::new(fixture.path()).parent().unwrap();
    let case: Case = serde_json::from_str(fixture.content()).expect("editor fixture schema");
    assert!(!case.documents.is_empty(), "fixture has no documents");
    assert!(!case.checks.is_empty(), "fixture has no checks");
    for check in case.checks {
        let world = open_documents(root, &case.documents);
        match check {
            Check::Rename(check) => {
                for position in positions(&world, &check.at) {
                    let mut fresh = open_documents(root, &case.documents);
                    check_rename(root, &case.documents, &mut fresh, &check, position);
                }
            }
            Check::RenameSequence { steps } => {
                assert!(!steps.is_empty(), "rename sequence has no steps");
                let mut world = world;
                for check in steps {
                    let positions = positions(&world, &check.at);
                    assert_eq!(positions.len(), 1, "sequential rename has one origin");
                    check_rename(root, &case.documents, &mut world, &check, positions[0]);
                }
            }
            Check::Completion {
                at,
                parse_clean,
                contains,
                excludes,
            } => {
                let target = uri(&at.document);
                if parse_clean {
                    let file = world
                        .db()
                        .source_file(world.vfs_path_for_uri(&target).unwrap())
                        .unwrap();
                    assert!(
                        parser::parse_file_to_hir(world.db(), file)
                            .diagnostics(world.db())
                            .is_empty()
                    );
                }
                for position in positions(&world, &at) {
                    let result =
                        crate::handle_completion(&world, &target, position).expect("completion");
                    let items = match result {
                        CompletionResponse::Array(items) => items,
                        CompletionResponse::List(list) => list.items,
                    };
                    for expected in &contains {
                        assert!(
                            items.iter().any(|item| item.label == expected.label
                                && item.kind == Some(expected.kind)),
                            "missing completion {}: {items:?}",
                            expected.label
                        );
                    }
                    for absent in &excludes {
                        assert!(
                            items.iter().all(|item| item.label != *absent),
                            "unexpected completion {absent}"
                        );
                    }
                }
            }
            Check::Signature {
                at,
                active_parameter,
            } => {
                for position in positions(&world, &at) {
                    let help = crate::handle_signature_help(&world, &uri(&at.document), position)
                        .expect("signature help");
                    assert_eq!(help.active_parameter, Some(active_parameter));
                }
            }
        }
    }
}
