use std::{collections::BTreeMap, path::Path};

use dir_test::{Fixture, dir_test};
use hir::ast::item::Item;
use nameres::{Db as _, LibraryId, ModuleKey, module_id_from_key, module_key_for_path};
use parser::parse_file_to_hir;
use serde::Deserialize;
use solcore_hir_ty::{
    ClassId, Evidence, Pred, Solution, Ty, canonical_goal, solve, trait_env_for_module,
};
use solcore_test_utils::{
    define_frontend_test_db, load_fixture_case, load_fixture_case_with_file_urls,
    load_reachable_modules, load_reachable_modules_with_file_urls,
};

define_frontend_test_db!(TestDb, solcore_hir_ty);

#[derive(Deserialize)]
#[serde(tag = "result", rename_all = "snake_case", deny_unknown_fields)]
enum Expected {
    Unique { goal: String, owners: Vec<String> },
    Ambiguous { goal: String, count: usize },
    NoSolution { goal: String },
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/default_precedence", glob: "**/expected.json")]
fn default_precedence(fixture: Fixture<&str>) {
    let case = Path::new(fixture.path()).parent().unwrap();
    let expected: Expected = serde_json::from_str(fixture.content()).unwrap();
    // Virtual libraries and overlapping physical roots must assign the same
    // priority to the same standard source, regardless of its logical alias.
    for (file_urls, overlapping) in [(false, false), (true, false), (true, true)] {
        let mut db = TestDb::default();
        let main_dir = case.join("main");
        let root = if overlapping { case } else { &main_dir };
        let mut external = BTreeMap::new();
        if case.join("external").exists() {
            external.insert("custom".to_owned(), case.join("external"));
        }
        if file_urls {
            load_fixture_case_with_file_urls(&mut db, root, case, external);
        } else {
            load_fixture_case(&mut db, root, case, external);
        }
        let entry = module_key_for_path(LibraryId::Main, root, &main_dir.join("main.sol")).unwrap();
        if file_urls {
            load_reachable_modules_with_file_urls(&mut db, entry.clone());
        } else {
            load_reachable_modules(&mut db, entry.clone());
        }
        let entry = module_id_from_key(&db, &entry);
        let diagnostics = solcore_hir_ty::collect_frontend_diagnostics(&db, entry);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let std_module = module_id_from_key(
            &db,
            &ModuleKey {
                library: LibraryId::Std,
                logical_path: vec!["std".to_owned()],
            },
        );
        let module = parse_file_to_hir(&db, db.module_file(std_module).unwrap()).module(&db);
        let (Expected::Unique { goal, .. }
        | Expected::Ambiguous { goal, .. }
        | Expected::NoSolution { goal }) = &expected;
        let class = module
            .items(&db)
            .iter()
            .find_map(|item| match item {
                Item::ClassDef(class)
                    if class.def_id_value(&db).name(&db).as_ref() == Some(goal) =>
                {
                    Some(ClassId::User(class.def_id_value(&db)))
                }
                _ => None,
            })
            .unwrap();
        let goal = canonical_goal(&db, Pred::in_class(&db, class, Ty::word(&db), vec![]));
        let actual = solve(&db, trait_env_for_module(&db, entry), goal);
        match (&expected, &actual) {
            (Expected::Unique { owners, .. }, Solution::Unique { evidence, .. }) => {
                let mut actual_owners = Vec::new();
                evidence_owners(&db, evidence, &mut actual_owners);
                assert_eq!(
                    &actual_owners, owners,
                    "{file_urls}/{overlapping}: {actual:?}"
                );
            }
            (Expected::Ambiguous { count, .. }, Solution::Ambiguous { candidates }) => {
                assert_eq!(candidates.len(), *count)
            }
            (Expected::NoSolution { .. }, Solution::NoSolution) => {}
            _ => panic!("{file_urls}/{overlapping}: {actual:?}"),
        }
    }
}

fn evidence_owners(db: &TestDb, evidence: &Evidence<'_>, owners: &mut Vec<String>) {
    match evidence {
        Evidence::Instance {
            instance,
            sub_evidence,
            ..
        } => {
            let url = instance.file(db).url(db);
            owners.push(
                Path::new(url.path())
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
            for child in sub_evidence {
                evidence_owners(db, child, owners);
            }
        }
        Evidence::Superclass { child, .. } => evidence_owners(db, child, owners),
        Evidence::Derived { sub_evidence, .. } => {
            for child in sub_evidence {
                evidence_owners(db, child, owners);
            }
        }
        Evidence::Builtin { .. } => {}
    }
}
