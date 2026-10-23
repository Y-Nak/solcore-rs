#![allow(dead_code)]

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use hir::{
    diag::{AnyDiagnostic, DiagnosticLevel},
    input::SourceFile,
};
use nameres::{
    LibraryId, ModuleFileSnapshot, ModuleFsSnapshot, ModuleId, ModuleKey, ModuleTree,
    module_id_from_key, module_key_for_path, module_path_display, reachable_diagnostics,
    resolve_module_path_candidate, resolve_reachable_full,
};
use parser::parse_file_to_hir;
use rustc_hash::{FxHashMap, FxHashSet};
use salsa::Setter;
use solcore_hir_ty::infer::reachable_typeck_diagnostics;

pub struct RunOutcome {
    pub unresolved_imports: Vec<String>,
    pub frontend_diagnostics: Vec<String>,
    pub frontend_error_diagnostics: Vec<String>,
    pub frontend_has_errors: bool,
    pub typeck_diagnostics: Vec<String>,
    pub typeck_error_diagnostics: Vec<String>,
    pub typeck_has_errors: bool,
    pub executed: Vec<String>,
}

#[salsa::db]
#[derive(Clone)]
struct TestDb {
    storage: salsa::Storage<Self>,
    module_tree: Option<ModuleTree>,
    module_fs_snapshot: Option<ModuleFsSnapshot>,
    module_file_snapshot: Option<ModuleFileSnapshot>,
    module_files: FxHashMap<ModuleKey, SourceFile>,
    executed: Arc<Mutex<Vec<String>>>,
}

impl Default for TestDb {
    fn default() -> Self {
        let executed = Arc::new(Mutex::new(Vec::new()));
        Self {
            storage: salsa::Storage::new(Some(Box::new({
                let executed = executed.clone();
                move |event| {
                    if let salsa::EventKind::WillExecute { database_key } = event.kind {
                        executed
                            .lock()
                            .expect("execution log lock")
                            .push(format!("{database_key:?}"));
                    }
                }
            }))),
            module_tree: None,
            module_fs_snapshot: None,
            module_file_snapshot: None,
            module_files: FxHashMap::default(),
            executed,
        }
    }
}

impl TestDb {
    fn take_executed(&self) -> Vec<String> {
        std::mem::take(&mut *self.executed.lock().expect("execution log lock"))
    }

    fn insert_module_file(&mut self, key: ModuleKey, file: SourceFile) {
        if self.module_files.insert(key, file) == Some(file) {
            return;
        }
        let files = self
            .module_files
            .iter()
            .map(|(key, file)| (key.clone(), *file))
            .collect();
        if let Some(snapshot) = self.module_file_snapshot {
            snapshot.set_files(self).to(files);
        } else {
            self.module_file_snapshot = Some(ModuleFileSnapshot::new(self, files));
        }
    }
}

#[salsa::db]
impl salsa::Database for TestDb {}

#[salsa::db]
impl hir::Db for TestDb {
    fn def_location_table<'db>(
        &'db self,
        file: SourceFile,
    ) -> &'db hir::anchor::DefLocationTable<'db> {
        parse_file_to_hir(self, file).def_locations(self)
    }
}

#[salsa::db]
impl parser::Db for TestDb {}

#[salsa::db]
impl nameres::Db for TestDb {
    fn module_tree(&self) -> ModuleTree {
        self.module_tree.expect("test module tree initialized")
    }

    fn module_fs_snapshot(&self) -> ModuleFsSnapshot {
        self.module_fs_snapshot
            .expect("test module filesystem snapshot initialized")
    }

    fn module_file_snapshot(&self) -> ModuleFileSnapshot {
        self.module_file_snapshot
            .expect("test module file snapshot initialized")
    }

    fn module_file<'db>(&'db self, module: ModuleId<'db>) -> Option<SourceFile> {
        self.module_file_snapshot()
            .files(self)
            .get(&module.key(self))
            .copied()
    }
}

#[salsa::db]
impl solcore_hir_ty::Db for TestDb {}

pub fn run_frontend(path: &Path, std_root: &Path) -> RunOutcome {
    let main_root = path
        .parent()
        .expect("entry path has a parent directory")
        .to_path_buf();
    run_frontend_with_roots(path, &main_root, std_root, BTreeMap::new())
}

pub fn run_frontend_with_roots(
    path: &Path,
    main_root: &Path,
    std_root: &Path,
    external_roots: BTreeMap<String, PathBuf>,
) -> RunOutcome {
    let mut db = test_db_for_roots(main_root, std_root, external_roots);
    run_frontend_in_db(&mut db, path, main_root)
}

fn test_db_for_roots(
    main_root: &Path,
    std_root: &Path,
    external_roots: BTreeMap<String, PathBuf>,
) -> TestDb {
    let mut db = TestDb::default();
    db.module_tree = Some(ModuleTree::new(
        &db,
        main_root.to_path_buf(),
        std_root.to_path_buf(),
        external_roots.clone(),
    ));
    db.module_fs_snapshot = Some(module_fs_snapshot_for_roots(
        &db,
        std::iter::once(main_root)
            .chain(std::iter::once(std_root))
            .chain(external_roots.values().map(|path| path.as_path())),
    ));
    db
}

fn insert_entry_source(db: &mut TestDb, path: &Path, main_root: &Path) -> ModuleKey {
    let entry_key = module_key_for_path(LibraryId::Main, main_root, path)
        .expect("entry file is under its main root");
    if !db.module_files.contains_key(&entry_key) {
        let source = fs::read_to_string(path).expect("fixture source");
        let entry_file = source_file_for_path(db, path, source);
        db.insert_module_file(entry_key.clone(), entry_file);
    }
    entry_key
}

fn run_frontend_in_db(db: &mut TestDb, path: &Path, main_root: &Path) -> RunOutcome {
    let entry_key = insert_entry_source(db, path, main_root);

    let unresolved_imports = load_reachable_modules(db, entry_key.clone());
    let entry = module_id_from_key(&*db, &entry_key);
    let _ = db.take_executed();
    let _ = resolve_reachable_full(&*db, entry);
    let reachable_frontend = reachable_diagnostics(&*db, entry);
    let mut frontend_error_diagnostics = summarize_error_diagnostics(&*db, reachable_frontend);
    frontend_error_diagnostics.extend(
        unresolved_imports
            .iter()
            .map(|unresolved| format!("unresolved-import: {unresolved}")),
    );
    frontend_error_diagnostics.sort();
    frontend_error_diagnostics.dedup();
    let frontend_has_errors = !frontend_error_diagnostics.is_empty();
    let mut frontend_diagnostics = summarize_diagnostics(&*db, reachable_frontend);
    frontend_diagnostics.extend(
        unresolved_imports
            .iter()
            .map(|unresolved| format!("unresolved-import: {unresolved}")),
    );
    frontend_diagnostics.sort();
    frontend_diagnostics.dedup();
    let reachable_typeck = reachable_typeck_diagnostics(&*db, entry);
    let typeck_error_diagnostics = summarize_error_diagnostics(&*db, reachable_typeck);
    let typeck_has_errors = !typeck_error_diagnostics.is_empty();
    let typeck_diagnostics = summarize_diagnostics(&*db, reachable_typeck);
    let executed = db.take_executed();

    RunOutcome {
        unresolved_imports,
        frontend_diagnostics,
        frontend_error_diagnostics,
        frontend_has_errors,
        typeck_diagnostics,
        typeck_error_diagnostics,
        typeck_has_errors,
        executed,
    }
}

fn load_reachable_modules(db: &mut TestDb, entry: ModuleKey) -> Vec<String> {
    let mut queue = VecDeque::from([entry]);
    let mut visited = FxHashSet::default();
    let mut unresolved = Vec::new();

    while let Some(key) = queue.pop_front() {
        if !visited.insert(key.clone()) {
            continue;
        }
        let Some(file) = db.module_files.get(&key).copied() else {
            continue;
        };
        let targets = {
            let module = module_id_from_key(&*db, &key);
            let refs = nameres::module_imports(&*db, file);
            refs.import_refs
                .into_iter()
                .chain(refs.export_refs)
                .filter_map(
                    |path| match resolve_module_path_candidate(&*db, module, &path) {
                        Ok(resolved) => Some((resolved.module.key(&*db), resolved.file_path)),
                        Err(_) => {
                            unresolved.push(format!(
                                "{} imports `{}`",
                                module.display(&*db),
                                module_path_display(&*db, &path)
                            ));
                            None
                        }
                    },
                )
                .collect::<Vec<_>>()
        };
        for (target_key, file_path) in targets {
            if !db.module_files.contains_key(&target_key) {
                match fs::read_to_string(&file_path) {
                    Ok(source) => {
                        let file = source_file_for_path(db, &file_path, source);
                        db.insert_module_file(target_key.clone(), file);
                    }
                    Err(err) => unresolved.push(format!(
                        "failed to read {} for {}: {err}",
                        file_path.display(),
                        module_key_display(&target_key)
                    )),
                }
            }
            if db.module_files.contains_key(&target_key) {
                queue.push_back(target_key);
            }
        }
    }

    unresolved.sort();
    unresolved.dedup();
    unresolved
}

fn module_fs_snapshot_for_roots<'a>(
    db: &TestDb,
    roots: impl IntoIterator<Item = &'a Path>,
) -> ModuleFsSnapshot {
    let mut existing_files = BTreeSet::new();
    let mut sibling_stems = BTreeMap::<PathBuf, BTreeSet<String>>::new();
    for root in roots {
        collect_module_fs_snapshot(root, &mut existing_files, &mut sibling_stems);
    }
    let sibling_stems = sibling_stems
        .into_iter()
        .map(|(parent, stems)| (parent, stems.into_iter().collect()))
        .collect();
    ModuleFsSnapshot::new(db, existing_files, sibling_stems)
}

fn collect_module_fs_snapshot(
    dir: &Path,
    existing_files: &mut BTreeSet<PathBuf>,
    sibling_stems: &mut BTreeMap<PathBuf, BTreeSet<String>>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("sol") {
            if path.is_file() {
                existing_files.insert(path.clone());
            }
            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                sibling_stems
                    .entry(dir.to_path_buf())
                    .or_default()
                    .insert(stem.to_owned());
            }
        }
        if path.is_dir() {
            collect_module_fs_snapshot(&path, existing_files, sibling_stems);
        }
    }
}

fn source_file_for_path(db: &TestDb, path: &Path, source: String) -> SourceFile {
    let url = url::Url::from_file_path(path).expect("file URL");
    SourceFile::new(db, url, Some(source))
}

fn summarize_diagnostics(db: &dyn hir::Db, diagnostics: &[AnyDiagnostic]) -> Vec<String> {
    let mut summaries = diagnostics
        .iter()
        .map(|diagnostic| {
            let diagnostic = diagnostic.lower(db);
            let code = diagnostic.code.as_deref().unwrap_or("no-code");
            format!("{code}: {}", diagnostic.message)
        })
        .collect::<Vec<_>>();
    summaries.sort();
    summaries.dedup();
    summaries
}

fn summarize_error_diagnostics(db: &dyn hir::Db, diagnostics: &[AnyDiagnostic]) -> Vec<String> {
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.lower(db).level == DiagnosticLevel::Error)
        .cloned()
        .collect::<Vec<_>>();
    summarize_diagnostics(db, &errors)
}
pub fn query_executions(events: &[String], query: &str) -> usize {
    events.iter().filter(|event| event.contains(query)).count()
}

fn module_key_display(key: &ModuleKey) -> String {
    let path = key.logical_path.join(".");
    match &key.library {
        LibraryId::Main => path,
        LibraryId::Std if key.logical_path.as_slice() == ["std"] => "std".to_owned(),
        LibraryId::Std => format!("std.{path}"),
        LibraryId::External(name) => format!("@{name}.{path}"),
    }
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("hir-ty crate lives under <repo>/crates/hir-ty")
        .to_path_buf()
}
