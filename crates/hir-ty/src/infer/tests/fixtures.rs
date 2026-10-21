use super::*;
use dir_test::{Fixture, dir_test};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    loader: Loader,
    #[serde(default)]
    nameres_empty: bool,
    functions: Vec<FunctionExpectation>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Loader {
    Bare,
    Project,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionExpectation {
    name: String,
    #[serde(default)]
    solve: bool,
    #[serde(default)]
    diagnostics_empty: bool,
    #[serde(default)]
    contains_codes: Vec<String>,
    #[serde(default)]
    absent_codes: Vec<String>,
    #[serde(default)]
    patterns_known: bool,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/inference_audit", glob: "**/expected.json")]
fn function_inference(fixture: Fixture<&str>) {
    let case_dir = PathBuf::from(fixture.path()).parent().unwrap().to_owned();
    let case: Case = serde_json::from_str(fixture.content()).expect("inference expectations");
    let mut db = TestDb::default();
    let main_path = PathBuf::from("/main/main.sol");
    let main_key =
        module_key_for_path(LibraryId::Main, &PathBuf::from("/main"), &main_path).unwrap();
    if let Loader::Project = case.loader {
        for entry in std::fs::read_dir(&case_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "sol") {
                let virtual_path = PathBuf::from("/main").join(path.file_name().unwrap());
                let key =
                    module_key_for_path(LibraryId::Main, &PathBuf::from("/main"), &virtual_path)
                        .unwrap();
                let file = source_file_at_path(
                    &db,
                    &virtual_path,
                    &std::fs::read_to_string(path).unwrap(),
                );
                db.insert_module_file(key, file);
            }
        }
    }
    let (module, module_id) = match case.loader {
        Loader::Bare => (
            parse_module(
                &db,
                &std::fs::read_to_string(case_dir.join("main.sol")).unwrap(),
            ),
            None,
        ),
        Loader::Project => (
            parse_file_to_hir(&db, db.module_files[&main_key]).module(&db),
            Some(module_id_from_key(&db, &main_key)),
        ),
    };
    if case.nameres_empty {
        let resolution = hir_nameres::resolve_module(&db, module);
        assert!(
            resolution.diagnostics.is_empty(),
            "{:?}",
            resolution.diagnostics
        );
    }
    for expected in case.functions {
        let (_, result) = if expected.solve {
            infer_module_function_with_solver(
                &db,
                module_id.expect("solver inference requires the project loader"),
                &expected.name,
            )
        } else {
            infer_function(&db, module, &expected.name)
        };
        if expected.diagnostics_empty {
            assert!(
                result.diagnostics.is_empty(),
                "{}: {:?}",
                expected.name,
                result.diagnostics
            );
        }
        let diagnostics = result
            .diagnostics
            .iter()
            .map(|d| d.lower())
            .collect::<Vec<_>>();
        for code in expected.contains_codes {
            assert!(
                diagnostics.iter().any(|d| d.code.as_ref() == Some(&code)),
                "{}: {diagnostics:#?}",
                expected.name
            );
        }
        for code in expected.absent_codes {
            assert!(
                diagnostics.iter().all(|d| d.code.as_ref() != Some(&code)),
                "{}: {diagnostics:#?}",
                expected.name
            );
        }
        if expected.patterns_known {
            for pat in &result.pat_tys {
                assert!(
                    !matches!(pat.ty.kind(&db), TyKind::Unknown | TyKind::Error),
                    "{}: {pat:?}",
                    expected.name
                );
            }
        }
    }
}
