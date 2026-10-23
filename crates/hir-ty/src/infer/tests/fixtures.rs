use super::*;
use dir_test::{Fixture, dir_test};
#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/inference_audit", glob: "**/main.sol")]
fn function_inference(fixture: Fixture<&str>) {
    let case_dir = PathBuf::from(fixture.path()).parent().unwrap().to_owned();
    let source = fixture.content();
    let directives = source
        .lines()
        .filter_map(|line| line.strip_prefix("//~ "))
        .collect::<Vec<_>>();
    let project = match directives
        .iter()
        .find_map(|line| line.strip_prefix("loader: "))
        .expect("fixture loader")
    {
        "bare" => false,
        "project" => true,
        other => panic!("unknown inference loader: {other}"),
    };
    let mut db = TestDb::default();
    let main_path = PathBuf::from("/main/main.sol");
    let main_key =
        module_key_for_path(LibraryId::Main, &PathBuf::from("/main"), &main_path).unwrap();
    if project {
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
    let (module, module_id) = if project {
        (
            parse_file_to_hir(&db, db.module_files[&main_key]).module(&db),
            Some(module_id_from_key(&db, &main_key)),
        )
    } else {
        (parse_module(&db, source), None)
    };
    if directives.contains(&"nameres-empty") {
        let resolution = hir_nameres::resolve_module(&db, module);
        assert!(
            resolution.diagnostics.is_empty(),
            "{:?}",
            resolution.diagnostics
        );
    }
    let mut checked_functions = 0;
    for directive in directives {
        if directive.starts_with("loader: ") || directive == "nameres-empty" {
            continue;
        }
        let value = directive
            .strip_prefix("infer: ")
            .unwrap_or_else(|| panic!("unknown inference expectation: {directive}"));
        let mut words = value.split_whitespace();
        let name = words.next().expect("function name");
        let flags = words.collect::<Vec<_>>();
        checked_functions += 1;
        let (_, result) = if flags.contains(&"solve") {
            infer_module_function_with_solver(
                &db,
                module_id.expect("solver inference requires the project loader"),
                name,
            )
        } else {
            infer_function(&db, module, name)
        };
        let diagnostics = result
            .diagnostics
            .iter()
            .map(|d| d.lower())
            .collect::<Vec<_>>();
        for flag in flags {
            match flag {
                "solve" => {}
                "diagnostics-empty" => assert!(diagnostics.is_empty(), "{name}: {diagnostics:#?}"),
                "patterns-known" => {
                    for pat in &result.pat_tys {
                        assert!(
                            !matches!(pat.ty.kind(&db), TyKind::Unknown | TyKind::Error),
                            "{name}: {pat:?}",
                        );
                    }
                }
                flag if flag.starts_with("contains-code=") => {
                    let code = flag.strip_prefix("contains-code=").unwrap();
                    assert!(
                        diagnostics.iter().any(|d| d.code.as_deref() == Some(code)),
                        "{name}: {diagnostics:#?}"
                    );
                }
                flag if flag.starts_with("absent-code=") => {
                    let code = flag.strip_prefix("absent-code=").unwrap();
                    assert!(
                        diagnostics.iter().all(|d| d.code.as_deref() != Some(code)),
                        "{name}: {diagnostics:#?}"
                    );
                }
                other => panic!("unknown inference expectation: {other}"),
            }
        }
    }
    assert!(
        checked_functions > 0,
        "fixture needs a function expectation"
    );
}
