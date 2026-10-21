use super::*;
use dir_test::{Fixture, dir_test};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    #[serde(default = "separate_root")]
    roots: Vec<RootLayout>,
    #[serde(default)]
    nameres_empty: bool,
    diagnostics: Option<DiagnosticExpectation>,
    surface: Option<SurfaceExpectation>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RootLayout {
    Separate,
    MainOnly,
    Shared,
}

fn separate_root() -> Vec<RootLayout> {
    vec![RootLayout::Separate]
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiagnosticExpectation {
    count: Option<usize>,
    all_code: Option<String>,
    #[serde(default)]
    contains: Vec<DiagnosticMatch>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiagnosticMatch {
    code: String,
    message: String,
    count: Option<usize>,
}

impl DiagnosticExpectation {
    fn check(&self, actual: &[Diagnostic]) {
        if let Some(count) = self.count {
            assert_eq!(actual.len(), count, "{actual:#?}");
        }
        if let Some(code) = &self.all_code {
            assert!(
                actual.iter().all(|d| d.code.as_ref() == Some(code)),
                "{actual:#?}"
            );
        }
        for expected in &self.contains {
            let count = actual
                .iter()
                .filter(|d| {
                    d.code.as_ref() == Some(&expected.code) && d.message.contains(&expected.message)
                })
                .count();
            if let Some(expected_count) = expected.count {
                assert_eq!(count, expected_count, "{}: {actual:#?}", expected.message);
            } else {
                assert!(count > 0, "{}: {actual:#?}", expected.message);
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SurfaceExpectation {
    contract: String,
    diagnostics: Option<DiagnosticExpectation>,
    methods: Option<Vec<MethodExpectation>>,
    constructor_inputs: Option<Vec<ParamExpectation>>,
    abi: AbiExpectation,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MethodExpectation {
    signature: String,
    selector: Option<String>,
    inputs: Vec<ParamExpectation>,
    outputs: Option<Vec<ParamExpectation>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamExpectation {
    ty: String,
    components: Vec<ParamExpectation>,
}

fn check_params(expected: &[ParamExpectation], actual: &[solcore_hir_ty::AbiParam]) {
    assert_eq!(actual.len(), expected.len(), "{actual:#?}");
    for (expected, actual) in expected.iter().zip(actual) {
        assert_eq!(actual.ty.to_string(), expected.ty);
        check_params(&expected.components, &actual.components);
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AbiExpectation {
    valid: bool,
    #[serde(default)]
    contains: Vec<String>,
    #[serde(default)]
    absent: Vec<String>,
    #[serde(default)]
    occurrences: BTreeMap<String, usize>,
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/contract_audit", glob: "**/expected.json")]
fn contract_semantics(fixture: Fixture<&str>) {
    let case_dir = PathBuf::from(fixture.path()).parent().unwrap().to_owned();
    let case: Case =
        serde_json::from_str(fixture.content()).expect("contract fixture expectations");
    let source = std::fs::read_to_string(case_dir.join("main.sol")).unwrap();
    for roots in case.roots {
        let (mut db, key) = match roots {
            RootLayout::Separate => {
                let (mut db, key) = db_with_main(&source);
                insert_real_std_modules(&mut db);
                (db, key)
            }
            RootLayout::MainOnly => db_with_overlapping_std(&source, false),
            RootLayout::Shared => db_with_overlapping_std(&source, true),
        };
        let project_root = match roots {
            RootLayout::Separate => "/main",
            RootLayout::MainOnly | RootLayout::Shared => "/std",
        };
        for entry in std::fs::read_dir(&case_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "sol")
                && path.file_name().unwrap() != "main.sol"
            {
                let name = path.file_stem().unwrap().to_str().unwrap();
                insert_module_source(
                    &mut db,
                    ModuleKey {
                        library: LibraryId::Main,
                        logical_path: vec![name.to_owned()],
                    },
                    &format!("{project_root}/{name}.sol"),
                    &std::fs::read_to_string(&path).unwrap(),
                );
            }
        }
        let module_id = module_id_from_key(&db, &key);
        if case.nameres_empty {
            let diagnostics = nameres::reachable_diagnostics(&db, module_id);
            assert!(diagnostics.is_empty(), "{roots:?}: {diagnostics:?}");
        }
        if let Some(expected) = &case.diagnostics {
            expected.check(&diagnostics_for_module(&db, &key));
        }
        if let Some(expected) = &case.surface {
            let module = parse_file_to_hir(&db, db.module_files[&key]).module(&db);
            let contract = contract_named(&db, module, &expected.contract);
            let surface = contract_dispatch_surface(&db, module, contract);
            if let Some(diagnostics) = &expected.diagnostics {
                diagnostics.check(&surface.diagnostics);
            }
            if let Some(methods) = &expected.methods {
                assert_eq!(surface.methods.len(), methods.len());
                for (actual, expected) in surface.methods.iter().zip(methods) {
                    assert_eq!(actual.signature, expected.signature);
                    if let Some(selector) = &expected.selector {
                        assert_eq!(&actual.selector.to_hex(), selector);
                    }
                    check_params(&expected.inputs, &actual.inputs);
                    if let Some(outputs) = &expected.outputs {
                        check_params(outputs, &actual.outputs);
                    }
                }
            }
            if let Some(expected_inputs) = &expected.constructor_inputs {
                let DispatchConstructor::Explicit { inputs, .. } = &surface.constructor else {
                    panic!("expected explicit constructor");
                };
                check_params(expected_inputs, inputs);
            }
            let abi = contract_abi_json(&db, module, contract);
            assert_eq!(abi.is_ok(), expected.abi.valid, "{abi:?}");
            if let Ok(abi) = abi {
                for text in &expected.abi.contains {
                    assert!(abi.contains(text), "{text}: {abi}");
                }
                for text in &expected.abi.absent {
                    assert!(!abi.contains(text), "{text}: {abi}");
                }
                for (text, count) in &expected.abi.occurrences {
                    assert_eq!(abi.matches(text.as_str()).count(), *count, "{text}: {abi}");
                }
            }
        }
    }
}
