//! Preserve semantic identities across a proposed workspace rename.
//!
//! Resolve both workspaces using the compiler's actual namespace, import,
//! lexical and expected-type rules. Compare source declarations through the
//! edit map rather than database IDs, which belong to separate Salsa databases.
//! Existing unresolved occurrences and diagnostics do not prevent an unrelated
//! rename, but a resolved reference may neither disappear nor change its target.

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
enum Meaning {
    Declaration {
        binding: SourceSpan,
        definition: SourceSpan,
    },
    Builtin(hir_nameres::BuiltinKind),
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct DiagnosticKey {
    code: Option<String>,
    labels: Vec<(bool, SourceSpan)>,
}

#[derive(Default)]
struct SemanticSnapshot {
    meanings: HashMap<SourceSpan, Meaning>,
    diagnostics: HashSet<DiagnosticKey>,
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

pub(crate) fn preserves_resolutions(
    world: &WorldState,
    edits: &HashMap<Url, Vec<TextEdit>>,
) -> bool {
    crate::analysis::with_analysis_stack(|| preserves_resolutions_inner(world, edits))
}

fn preserves_resolutions_inner(world: &WorldState, edits: &HashMap<Url, Vec<TextEdit>>) -> bool {
    let Some(before) = semantic_snapshot(world) else {
        return false;
    };
    let Some(edits) = ProposedEdits::new(world, edits) else {
        return false;
    };
    let mut proposed = world.fork();
    if edits.apply(&mut proposed).is_none() {
        return false;
    }
    let Some(after) = semantic_snapshot(&proposed) else {
        return false;
    };
    let Some(previous_diagnostics) = before
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            Some(DiagnosticKey {
                code: diagnostic.code,
                labels: diagnostic
                    .labels
                    .into_iter()
                    .map(|(primary, span)| Some((primary, edits.map_span(&span)?)))
                    .collect::<Option<_>>()?,
            })
        })
        .collect::<Option<HashSet<_>>>()
    else {
        return false;
    };
    if !after.diagnostics.is_subset(&previous_diagnostics) {
        return false;
    }
    before.meanings.into_iter().all(|(span, meaning)| {
        let meaning = match meaning {
            Meaning::Declaration {
                binding,
                definition,
            } => {
                let (Some(binding), Some(definition)) =
                    (edits.map_span(&binding), edits.map_span(&definition))
                else {
                    return false;
                };
                Meaning::Declaration {
                    binding,
                    definition,
                }
            }
            meaning => meaning,
        };
        edits
            .map_span(&span)
            .is_some_and(|span| after.meanings.get(&span) == Some(&meaning))
    })
}

impl SemanticSnapshot {
    fn record<'db>(
        &mut self,
        world: &'db WorldState,
        span: Span<'db>,
        resolution: &Resolution<'db>,
    ) -> Option<()> {
        let db = world.db();
        let meaning = match resolution {
            Resolution::Builtin(builtin) => Meaning::Builtin(*builtin),
            Resolution::DotCtorDeferred | Resolution::Err => return Some(()),
            _ => {
                // Keep the local alias and its provider as separate identities:
                // preserving an alias declaration alone does not preserve what
                // an import with that alias means.
                let definition = target_from_resolution(resolution)?;
                let binding = target_from_resolution_at_span(db, resolution, span)?;
                let (Some(binding), Some(definition)) = (
                    target_declaration_span(db, &binding),
                    target_declaration_span(db, &definition),
                ) else {
                    // Some implicit module qualifiers have no source declaration.
                    // Their selected members still have stable identities.
                    return Some(());
                };
                Meaning::Declaration {
                    binding: SourceSpan::new(world, binding)?,
                    definition: SourceSpan::new(world, definition)?,
                }
            }
        };
        self.meanings.insert(
            SourceSpan::new(world, span.resolve_to_absolute(db))?,
            meaning,
        );
        Some(())
    }

    fn record_qualifier<'db>(
        &mut self,
        world: &'db WorldState,
        env: &dyn hir_nameres::ImportedNames<'db>,
        qualifier: &SpannedElem<'db, hir::ast::Ident<'db>>,
    ) -> Option<()> {
        let db = world.db();
        for (target, span) in module_qualifier_occurrences(db, env, qualifier) {
            if let Some(declaration) = target_declaration_span(db, &target) {
                let declaration = SourceSpan::new(world, declaration)?;
                self.meanings.insert(
                    SourceSpan::new(world, span.resolve_to_absolute(db))?,
                    Meaning::Declaration {
                        binding: declaration.clone(),
                        definition: declaration,
                    },
                );
            }
        }
        Some(())
    }

    fn record_types<'db>(
        &mut self,
        world: &'db WorldState,
        env: &dyn hir_nameres::ImportedNames<'db>,
        types: &[hir_nameres::TypeResolution<'db>],
        preds: &[hir_nameres::PredResolution<'db>],
    ) -> Option<()> {
        let db = world.db();
        for entry in types {
            if let Some(qualifier) = type_ref_module_qualifier(db, entry.ty) {
                self.record_qualifier(world, env, qualifier)?;
            }
            if let Some(span) = type_ref_name_span(db, entry.ty) {
                self.record(world, span, &entry.resolution)?;
            }
        }
        for entry in preds {
            if let Some(span) = pred_ref_class_span(db, entry.pred) {
                self.record(world, span, &entry.resolution)?;
            }
        }
        Some(())
    }
}

fn semantic_snapshot(world: &WorldState) -> Option<SemanticSnapshot> {
    let db = world.db();
    let mut snapshot = SemanticSnapshot::default();
    for module_id in reference_search_modules(world, db) {
        let Some(file) = db.module_file(module_id) else {
            continue;
        };
        let module = parser::parse_file_to_hir(db, file).module(db);
        let env = nameres::module_env(db, module_id);
        let map =
            module_reference_resolutions(db, module, hir_nameres::item_scope(db, module), &env);
        // Comparing diagnostic code and source identities also rejects unused
        // declaration/import collisions, without blocking pre-existing errors.
        for diagnostic in nameres::module_diagnostics(db, module_id) {
            let diagnostic = diagnostic.lower(db);
            if diagnostic.level != hir::diag::DiagnosticLevel::Error {
                continue;
            }
            snapshot.diagnostics.insert(DiagnosticKey {
                code: diagnostic.code,
                labels: diagnostic
                    .labels
                    .iter()
                    .map(|label| {
                        Some((
                            label.is_primary(),
                            SourceSpan::new(world, label.span().resolve_to_absolute(db))?,
                        ))
                    })
                    .collect::<Option<_>>()?,
            });
        }
        snapshot.record_types(
            world,
            &env,
            &map.item_resolutions.facts.types,
            &map.item_resolutions.facts.preds,
        )?;
        for body in &map.bodies {
            snapshot.record_types(world, &env, &body.types, &body.preds)?;
            for entry in &body.exprs {
                if let Some(span) = expr_reference_span(db, entry.body.exprs(db).get(entry.expr)) {
                    snapshot.record(world, span, &entry.resolution)?;
                }
            }
            for entry in &body.pats {
                let pat = entry.body.pats(db).get(entry.pat);
                if let Some(qualifier) = pat_module_qualifier(pat) {
                    snapshot.record_qualifier(world, &env, qualifier)?;
                }
                if let Some(span) = pat_reference_span(db, pat) {
                    snapshot.record(world, span, &entry.resolution)?;
                }
            }
            for entry in &body.assembly {
                snapshot.record(world, entry.span, &entry.resolution)?;
            }
            for entry in &body.stmt_bindings {
                if let Some(span) = stmt_binding_span(db, entry.body, entry.stmt) {
                    snapshot.record(world, span, &entry.resolution)?;
                }
            }
        }
    }
    Some(snapshot)
}
