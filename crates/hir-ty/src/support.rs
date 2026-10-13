use hir::{
    anchor::DefId,
    ast::item::{Item, Module},
};
use nameres::{
    LibraryId, ModuleId, module_id_for_source_file, module_id_from_key, module_key_for_path,
    reachable_modules,
};
use parser::parse_file_to_hir;

use crate::{Db, Ty, TyCtor, UserTyCtor, UserTyCtorKind};

pub(crate) fn canonical_std_module_ids<'db>(
    db: &'db dyn Db,
    name: &str,
) -> impl Iterator<Item = ModuleId<'db>> {
    let std_module = module_id_from_key(
        db,
        &nameres::ModuleKey {
            library: LibraryId::Std,
            logical_path: vec![name.to_owned()],
        },
    );
    let tree = db.module_tree();
    let main_module = module_key_for_path(
        LibraryId::Main,
        tree.main_root(db),
        &tree.std_root(db).join(format!("{name}.sol")),
    )
    .map(|key| module_id_from_key(db, &key));
    std::iter::once(std_module).chain(main_module)
}

pub(crate) fn canonical_std_adt_defs<'db>(db: &'db dyn Db, name: &str) -> Vec<DefId<'db>> {
    let mut defs = canonical_std_module_ids(db, "std")
        .filter_map(|module| {
            let file = db.module_file(module)?;
            parse_file_to_hir(db, file)
                .module(db)
                .items(db)
                .iter()
                .find_map(|item| match item {
                    Item::AdtDef(adt) if adt.def_id_value(db).name(db).as_deref() == Some(name) => {
                        Some(adt.def_id_value(db))
                    }
                    _ => None,
                })
        })
        .collect::<Vec<_>>();
    defs.dedup();
    defs
}

pub(crate) fn canonical_std_adt_def<'db>(db: &'db dyn Db, name: &str) -> Option<DefId<'db>> {
    canonical_std_adt_defs(db, name).into_iter().next()
}

pub(crate) fn source_string_ty<'db>(db: &'db dyn Db) -> Ty<'db> {
    canonical_std_adt_def(db, "string")
        .map(|def| {
            Ty::named(
                db,
                TyCtor::User(UserTyCtor {
                    def,
                    kind: UserTyCtorKind::Adt,
                }),
                Vec::new(),
            )
        })
        .unwrap_or_else(|| Ty::string(db))
}

pub(crate) fn source_string_ty_for_module<'db>(db: &'db dyn Db, module: Module<'db>) -> Ty<'db> {
    module
        .items(db)
        .iter()
        .find_map(|item| match item {
            Item::AdtDef(adt)
                if adt.def_id_value(db).name(db).as_deref() == Some("string")
                    && is_canonical_std_def_named(db, adt.def_id_value(db), "string") =>
            {
                Some(adt.def_id_value(db))
            }
            _ => None,
        })
        .map(|def| {
            Ty::named(
                db,
                TyCtor::User(UserTyCtor {
                    def,
                    kind: UserTyCtorKind::Adt,
                }),
                Vec::new(),
            )
        })
        .unwrap_or_else(|| source_string_ty(db))
}

pub(crate) fn module_for_def_via_graph<'db>(
    db: &'db dyn Db,
    entry: ModuleId<'db>,
    def: DefId<'db>,
) -> Option<ModuleId<'db>> {
    let file = def.file(db);
    reachable_modules(db, entry)
        .into_iter()
        .find(|module| db.module_file(*module) == Some(file))
}

pub(crate) fn module_for_def_via_tree<'db>(
    db: &'db dyn Db,
    def: DefId<'db>,
) -> Option<ModuleId<'db>> {
    let path = hir::url_to_file_path(def.file(db).url(db))?;
    let tree = db.module_tree();
    let candidates = std::iter::once((LibraryId::Main, tree.main_root(db).clone()))
        .chain(std::iter::once((LibraryId::Std, tree.std_root(db).clone())))
        .chain(
            tree.external_roots(db)
                .iter()
                .map(|(name, root)| (LibraryId::External(name.clone()), root.clone())),
        );
    for (library, root) in candidates {
        if let Some(key) = module_key_for_path(library, &root, &path) {
            return Some(module_id_from_key(db, &key));
        }
    }
    None
}

pub fn is_canonical_std_def_named(db: &dyn Db, def: DefId<'_>, name: &str) -> bool {
    is_canonical_std_module_def_named(db, def, "std", name)
}

/// Recognizes a definition from a canonical std file even when overlapping roots
/// make that file reachable through the main library.
pub fn is_canonical_std_module_def_named(
    db: &dyn Db,
    def: DefId<'_>,
    module_name: &str,
    name: &str,
) -> bool {
    if def.name(db).as_deref() != Some(name) {
        return false;
    }

    let file = def.file(db);
    let tree = db.module_tree();
    let is_std_root_file = hir::url_to_file_path(file.url(db))
        .and_then(|path| module_key_for_path(LibraryId::Std, tree.std_root(db), &path))
        .is_some_and(|key| key.logical_path.as_slice() == [module_name]);

    is_std_root_file
        || module_id_for_source_file(db, file).is_some_and(|module| {
            module.library(db) == &LibraryId::Std
                && module.logical_path(db).as_slice() == [module_name]
        })
}
