//! Preserve lexical binding identities when renaming a source variable.
//!
//! Name resolution already records each declaration's scope and introduction
//! point. Replaying lookup over those facts checks both the edited references
//! and references that the renamed declaration could newly capture, without
//! maintaining another source or assembly scope walker in the LSP.

use super::*;

pub(super) fn preserves_local_bindings<'db>(
    db: &'db dyn hir_ty::Db,
    body: &hir_nameres::BodyResolutionMap<'db>,
    target: &ReferenceTarget<'db>,
    new_name: &str,
) -> bool {
    let target = match target {
        ReferenceTarget::Param(param) => Resolution::Param(*param),
        ReferenceTarget::Local(
            local @ (LocalBinding::Let { .. } | LocalBinding::Pattern { .. }),
        ) => Resolution::Local(local.clone()),
        _ => return true,
    };
    if !body
        .scoped_bindings
        .iter()
        .any(|binding| binding.resolution == target)
    {
        return true;
    }

    let preserves = |name: &str, span: Span<'db>, resolution: &Resolution<'db>| {
        let edited = resolution == &target;
        if !edited && name != new_name {
            return true;
        }
        let name = if edited { new_name } else { name };
        let occurrence = span.resolve_to_absolute(db);
        let selected = body.scoped_bindings.iter().rev().find(|binding| {
            let binding_name = if binding.resolution == target {
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
            // An edited local reference must still resolve to its declaration.
            // An unaffected item/field reference remains outside local lookup.
            None => !edited,
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
