//! Preserve lexical binding identities when renaming a source symbol.
//!
//! Name resolution already records each declaration's scope and introduction
//! point. Replaying lookup over those facts checks both the edited references
//! and references that the renamed declaration could newly capture, without
//! maintaining another source or assembly scope walker in the LSP.

use super::*;

pub(super) fn preserves_bindings<'db>(
    db: &'db dyn hir_ty::Db,
    body: &hir_nameres::BodyResolutionMap<'db>,
    target: &ReferenceTarget<'db>,
    new_name: &str,
) -> bool {
    let preserves = |name: &str, span: Span<'db>, resolution: &Resolution<'db>| {
        // Import aliases share the provider's resolution, but rename only
        // edits occurrences belonging to that alias.
        let edited = resolution_matches_target(db, span, resolution, target);
        if !edited && name != new_name {
            return true;
        }
        let name = if edited { new_name } else { name };
        let occurrence = span.resolve_to_absolute(db);
        let selected = body.scoped_bindings.iter().rev().find(|binding| {
            let binding_name =
                if target_from_resolution(&binding.resolution).as_ref() == Some(target) {
                    new_name
                } else {
                    binding.name.as_str()
                };
            if binding_name != name {
                return false;
            }
            let scope = binding.scope.resolve_to_absolute(db);
            let introduced = binding.introduced_after.resolve_to_absolute(db);
            scope.file() == occurrence.file()
                && scope.start() <= occurrence.start()
                && occurrence.start() < scope.end()
                && introduced.file() == occurrence.file()
                && introduced.end() <= occurrence.start()
        });
        match selected {
            Some(binding) => &binding.resolution == resolution,
            // Parameters and locals must still select their declaration.
            // Items, fields and module qualifiers resolve outside local lookup
            // but may not acquire a local binding after their name is edited.
            None => !matches!(
                resolution,
                Resolution::Param(_)
                    | Resolution::Local(LocalBinding::Let { .. } | LocalBinding::Pattern { .. })
            ),
        }
    };

    body.exprs.iter().all(|entry| {
        let expr = entry.body.exprs(db).get(entry.expr);
        let ExprKind::Ident(name) = &expr.kind else {
            // Qualified members and constructor syntax do not perform local
            // lookup. Their qualifier expressions are checked independently.
            return true;
        };
        preserves(name.atom().text(db), name.span(db), &entry.resolution)
    }) && body.assembly.iter().all(|entry| {
        // Assembly facts contain only references crossing into source scopes;
        // Yul locals and non-capturing function bodies cannot be affected.
        let span = entry.span.resolve_to_absolute(db);
        let Some(text) = span.file().content(db).as_deref() else {
            return false;
        };
        let Some(name) = text.get(span.start().as_u32() as usize..span.end().as_u32() as usize)
        else {
            return false;
        };
        preserves(name, entry.span, &entry.resolution)
    })
}
