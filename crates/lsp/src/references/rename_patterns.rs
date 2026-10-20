//! Validate pattern roles against the actual renamed module surfaces.
//!
//! Pattern names resolve before lexical binding: an item or import rename can
//! turn an untouched catch-all into a constructor, and a binder rename can do
//! the same to the edited declaration. Re-resolve the proposed workspace in an
//! independent database instead of duplicating constructor visibility rules.

use std::collections::HashMap;

use lsp_types::TextEdit;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SourceSpan {
    uri: Url,
    start: u32,
    end: u32,
}

impl SourceSpan {
    fn new(world: &WorldState, span: AbsoluteSpan) -> Option<Self> {
        Some(Self {
            uri: world.client_uri_for_vfs_url(span.file().url(world.db()).as_str())?,
            start: span.start().as_u32(),
            end: span.end().as_u32(),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PatternMeaning {
    Binding(SourceSpan),
    Constructor(SourceSpan),
    Builtin(hir_nameres::BuiltinKind),
    Deferred,
    Unresolved,
}

struct ByteEdit<'a> {
    start: u32,
    end: u32,
    new_text: &'a str,
}

struct ProposedEdits<'a>(HashMap<Url, Vec<ByteEdit<'a>>>);

impl<'a> ProposedEdits<'a> {
    fn new(world: &WorldState, edits: &'a HashMap<Url, Vec<TextEdit>>) -> Option<Self> {
        let mut files = HashMap::new();
        for (uri, edits) in edits {
            let index = world.line_index(uri)?;
            let mut edits = edits
                .iter()
                .map(|edit| {
                    Some(ByteEdit {
                        start: index.position_to_byte(edit.range.start)?,
                        end: index.position_to_byte(edit.range.end)?,
                        new_text: edit.new_text.as_str(),
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            edits.sort_by_key(|edit| edit.start);
            files.insert(uri.clone(), edits);
        }
        Some(Self(files))
    }

    fn map_span(&self, span: &SourceSpan) -> Option<SourceSpan> {
        let Some(edits) = self.0.get(&span.uri) else {
            return Some(span.clone());
        };
        let offset = |old: u32| {
            let mut delta = 0_i64;
            for edit in edits {
                if old <= edit.start {
                    break;
                }
                if old < edit.end {
                    // Semantic names must lie outside edits or exactly match
                    // their ranges. An interior offset has no stable identity.
                    return None;
                }
                delta += edit.new_text.len() as i64 - i64::from(edit.end - edit.start);
            }
            u32::try_from(i64::from(old) + delta).ok()
        };
        Some(SourceSpan {
            uri: span.uri.clone(),
            start: offset(span.start)?,
            end: offset(span.end)?,
        })
    }

    fn apply(&self, world: &mut WorldState) -> Option<()> {
        for (uri, edits) in &self.0 {
            let mut text = world.line_index(uri)?.text().to_owned();
            for edit in edits.iter().rev() {
                text.replace_range(edit.start as usize..edit.end as usize, edit.new_text);
            }
            if !world.set_workspace_document(uri.clone(), text) {
                return None;
            }
        }
        Some(())
    }
}

pub(crate) fn preserves_pattern_resolutions(
    world: &WorldState,
    edits: &HashMap<Url, Vec<TextEdit>>,
) -> bool {
    let Some(before) = pattern_meanings(world) else {
        return false;
    };
    if before.is_empty() {
        return true;
    }
    let Some(edits) = ProposedEdits::new(world, edits) else {
        return false;
    };
    let mut proposed = world.fork();
    if edits.apply(&mut proposed).is_none() {
        return false;
    }
    let Some(after) = pattern_meanings(&proposed) else {
        return false;
    };
    before.into_iter().all(|(span, meaning)| {
        let meaning = match meaning {
            PatternMeaning::Binding(declaration) => {
                let Some(declaration) = edits.map_span(&declaration) else {
                    return false;
                };
                PatternMeaning::Binding(declaration)
            }
            PatternMeaning::Constructor(declaration) => {
                let Some(declaration) = edits.map_span(&declaration) else {
                    return false;
                };
                PatternMeaning::Constructor(declaration)
            }
            // An already unresolved occurrence has no binding to preserve.
            PatternMeaning::Unresolved => return true,
            meaning => meaning,
        };
        edits
            .map_span(&span)
            .is_some_and(|span| after.get(&span) == Some(&meaning))
    })
}

fn pattern_meanings(world: &WorldState) -> Option<HashMap<SourceSpan, PatternMeaning>> {
    let db = world.db();
    let mut meanings = HashMap::new();
    for module_id in reference_search_modules(world, db) {
        let Some(file) = db.module_file(module_id) else {
            continue;
        };
        let module = parser::parse_file_to_hir(db, file).module(db);
        let env = nameres::module_env(db, module_id);
        let map = hir_nameres::resolve_module_with_imports_and_policy(
            db,
            module,
            hir_nameres::item_scope(db, module),
            &env,
            hir_nameres::NameresDiagnosticPolicy::Emit,
        );
        for body in &map.bodies {
            for entry in &body.pats {
                let pat = entry.body.pats(db).get(entry.pat);
                let Some(span) = pat_reference_span(db, pat) else {
                    continue;
                };
                let meaning = match &entry.resolution {
                    Resolution::Local(LocalBinding::Pattern { .. }) | Resolution::Ctor { .. } => {
                        let target = target_from_resolution(&entry.resolution)?;
                        let declaration =
                            SourceSpan::new(world, target_declaration_span(db, &target)?)?;
                        if matches!(&entry.resolution, Resolution::Local(_)) {
                            PatternMeaning::Binding(declaration)
                        } else {
                            PatternMeaning::Constructor(declaration)
                        }
                    }
                    Resolution::Builtin(builtin) => PatternMeaning::Builtin(*builtin),
                    Resolution::DotCtorDeferred => PatternMeaning::Deferred,
                    _ => PatternMeaning::Unresolved,
                };
                meanings.insert(
                    SourceSpan::new(world, span.resolve_to_absolute(db))?,
                    meaning,
                );
            }
        }
    }
    Some(meanings)
}
