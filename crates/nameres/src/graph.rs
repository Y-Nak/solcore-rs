use super::*;

/// Extracts import and export module references from a source file.
///
/// The parser/lowerer owns syntax diagnostics; this query only classifies the
/// lowered import/export items for graph construction.
#[salsa::tracked(returns(clone))]
#[tracing::instrument(
    target = "nameres::query",
    level = "debug",
    skip(db, file),
    fields(file = field::Empty)
)]
pub fn module_imports<'db>(db: &'db dyn Db, file: SourceFile) -> ModuleImports<'db> {
    record_source_file_field(db, file);
    let module = parse_file_to_hir(db, file).module(db);
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut import_refs = Vec::new();
    let mut export_refs = Vec::new();

    for item in module.items(db) {
        match item {
            Item::Import(import) => {
                imports.push(*import);
                import_refs.push(path_ref_from_import(db, *import));
            }
            Item::Export(export) => {
                exports.push(*export);
                export_refs.extend(path_refs_from_export(db, *export));
            }
            _ => {}
        }
    }

    // Array syntax uses compiler-owned runtime helpers. Load their defining
    // module and instance evidence without opening any source-level names.
    if let Some(span) = array_literal_span(db, module)
        && !import_refs
            .iter()
            .any(|path| path.external.is_none() && path_segments(db, path) == ["std"])
    {
        import_refs.push(path_ref_from_text(db, span, "std"));
    }

    ModuleImports {
        imports,
        exports,
        import_refs,
        export_refs,
    }
}

fn array_literal_span<'db>(db: &'db dyn Db, module: Module<'db>) -> Option<Span<'db>> {
    use hir::ast::function::{Expr, ExprKind};

    fn expression<'db>(db: &'db dyn Db, expr: &Expr<'db>) -> Option<Span<'db>> {
        match &expr.kind {
            ExprKind::Array(_) => Some(expr.span),
            ExprKind::Lambda { body, .. } => body_span(db, *body),
            _ => None,
        }
    }
    fn body_span<'db>(db: &'db dyn Db, body: FuncBody<'db>) -> Option<Span<'db>> {
        body.exprs(db)
            .iter()
            .find_map(|(_, expr)| expression(db, expr))
    }
    fn function_span<'db>(db: &'db dyn Db, function: FunctionDef<'db>) -> Option<Span<'db>> {
        function.body(db).and_then(|body| body_span(db, body))
    }
    module.items(db).iter().find_map(|item| match item {
        Item::FunctionDef(function) => function_span(db, *function),
        Item::InstanceDef(instance) => instance
            .methods(db)
            .iter()
            .find_map(|f| function_span(db, *f)),
        Item::ContractDef(contract) => contract
            .fields(db)
            .iter()
            .find_map(|field| {
                field
                    .init()
                    .and_then(|init| init.exprs.iter().find_map(|(_, expr)| expression(db, expr)))
            })
            .or_else(|| {
                contract.items(db).iter().find_map(|item| match item {
                    ContractItem::FunctionDef(function) => function_span(db, *function),
                    _ => None,
                })
            }),
        _ => None,
    })
}

/// Builds the import/export reachability graph from `entry`.
///
/// Import edges represent direct imports. Reference edges include both imports
/// and module references that appear in exports/re-exports, because those also
/// participate in public-interface cycles.
#[salsa::tracked(returns(clone))]
pub fn module_graph<'db>(db: &'db dyn Db, entry: ModuleId<'db>) -> ModuleGraph<'db> {
    let reachable = collect_reachable_modules(db, entry);
    ModuleGraph {
        entry,
        modules: reachable.modules,
        import_edges: reachable.import_edges,
        reference_edges: reachable.reference_edges,
    }
}

/// Returns modules reachable from `entry` in the same traversal order as
/// [`module_graph`].
#[salsa::tracked(returns(clone))]
pub fn reachable_modules<'db>(db: &'db dyn Db, entry: ModuleId<'db>) -> Vec<ModuleId<'db>> {
    collect_reachable_modules(db, entry).modules
}

struct ReachableModules<'db> {
    modules: Vec<ModuleId<'db>>,
    import_edges: Vec<ModuleEdge<'db>>,
    reference_edges: Vec<ModuleEdge<'db>>,
}

fn collect_reachable_modules<'db>(db: &'db dyn Db, entry: ModuleId<'db>) -> ReachableModules<'db> {
    let mut modules = Vec::new();
    let mut seen = FxHashSet::default();
    let mut queue = VecDeque::from([entry]);
    let mut import_edges = Vec::new();
    let mut reference_edges = Vec::new();

    while let Some(module) = queue.pop_front() {
        if !seen.insert(module) {
            continue;
        }
        modules.push(module);

        let Some(file) = db.module_file(module) else {
            continue;
        };
        let refs = module_imports(db, file);

        for path in refs.import_refs {
            if let Ok(target) = resolve_module_path(db, module, path) {
                import_edges.push(ModuleEdge {
                    from: module,
                    to: target,
                });
                reference_edges.push(ModuleEdge {
                    from: module,
                    to: target,
                });
                queue.push_back(target);
            }
        }

        for path in refs.export_refs {
            if let Ok(target) = resolve_module_path(db, module, path) {
                reference_edges.push(ModuleEdge {
                    from: module,
                    to: target,
                });
                queue.push_back(target);
            }
        }
    }

    ReachableModules {
        modules,
        import_edges,
        reference_edges,
    }
}

/// Runs full resolution for every module reachable from `entry`.
#[salsa::tracked(returns(clone))]
pub fn resolve_reachable_full<'db>(db: &'db dyn Db, entry: ModuleId<'db>) -> ModuleGraph<'db> {
    let graph = module_graph(db, entry);
    for module in &graph.modules {
        let _ = resolve_module_full(db, *module);
    }
    graph
}
