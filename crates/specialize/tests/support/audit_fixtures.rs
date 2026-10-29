use dir_test::{Fixture, dir_test};

use super::*;

#[derive(Default)]
struct Expectations {
    loader: Loader,
    frontend_clean: Option<bool>,
    max_depth: Option<usize>,
    diagnostics: Vec<DiagnosticCheck>,
    forbidden_diagnostics: Vec<DiagnosticKind>,
    checks: Vec<Check>,
}

#[derive(Default)]
enum Loader {
    #[default]
    Bare,
    Std,
    Fixture,
}

struct DiagnosticCheck {
    kind: DiagnosticKind,
    context: Option<String>,
    context_contains: Option<String>,
}

enum DiagnosticKind {
    FreeTypeVariable,
    ComptimeEvaluationFailed,
    IntegerErasure,
    MissingEvidence,
    InstantiationDepthExceeded,
}

impl DiagnosticKind {
    fn matches(&self, actual: &SpecializeDiagnosticKind) -> bool {
        matches!(
            (self, actual),
            (
                Self::FreeTypeVariable,
                SpecializeDiagnosticKind::FreeTypeVariable { .. }
            ) | (
                Self::ComptimeEvaluationFailed,
                SpecializeDiagnosticKind::ComptimeEvaluationFailed { .. }
            ) | (
                Self::IntegerErasure,
                SpecializeDiagnosticKind::IntegerErasure { .. }
            ) | (
                Self::MissingEvidence,
                SpecializeDiagnosticKind::MissingEvidence { .. }
            ) | (
                Self::InstantiationDepthExceeded,
                SpecializeDiagnosticKind::InstantiationDepthExceeded { .. }
            )
        )
    }
}

enum Check {
    MainReturnNumber { value: Option<String> },
    MainReturnVariable { name: String },
    EntryLastReturnNumber { value: String },
    EntryLastReturnVariable { name: String },
    EntryExists,
    NoClosureDispatch,
}

impl DiagnosticKind {
    fn parse(name: &str) -> Self {
        match name {
            "free-type-variable" => Self::FreeTypeVariable,
            "comptime-evaluation-failed" => Self::ComptimeEvaluationFailed,
            "integer-erasure" => Self::IntegerErasure,
            "missing-evidence" => Self::MissingEvidence,
            "instantiation-depth-exceeded" => Self::InstantiationDepthExceeded,
            _ => panic!("unknown specialization diagnostic {name}"),
        }
    }
}

impl Expectations {
    fn from_source(source: &str) -> Option<Self> {
        let mut expectations = Self::default();
        let mut loader = None;
        let mut found = false;
        for (line, directive) in solcore_test_utils::source_directives(source, "specialize") {
            found = true;
            let (key, value) = directive.split_once(' ').unwrap_or((directive, ""));
            let value = value.trim();
            match (key, value) {
                ("loader", value) => {
                    assert!(loader.is_none(), "line {line}: duplicate loader");
                    loader = Some(match value {
                        "bare" => Loader::Bare,
                        "std" | "std-current" => Loader::Std,
                        "fixture" => Loader::Fixture,
                        _ => panic!("line {line}: unknown loader {value}"),
                    });
                }
                ("frontend", "clean" | "errors") => {
                    assert!(
                        expectations.frontend_clean.is_none(),
                        "line {line}: duplicate frontend expectation"
                    );
                    expectations.frontend_clean = Some(value == "clean");
                }
                ("max-depth", value) => {
                    assert!(
                        expectations.max_depth.is_none(),
                        "line {line}: duplicate max-depth"
                    );
                    expectations.max_depth = Some(value.parse().expect("specialization max-depth"));
                }
                ("diagnostic", value) => {
                    let (kind, context) = value.split_once(' ').unwrap_or((value, ""));
                    let mut check = DiagnosticCheck {
                        kind: DiagnosticKind::parse(kind),
                        context: None,
                        context_contains: None,
                    };
                    if let Some(context) = context.strip_prefix("= ") {
                        check.context = Some(context.to_owned());
                    } else if let Some(context) = context.strip_prefix("contains ") {
                        check.context_contains = Some(context.to_owned());
                    } else {
                        assert!(
                            context.is_empty(),
                            "line {line}: invalid diagnostic context {context}"
                        );
                    }
                    expectations.diagnostics.push(check);
                }
                ("forbid", value) => expectations
                    .forbidden_diagnostics
                    .push(DiagnosticKind::parse(value)),
                ("return", value) if !value.is_empty() => {
                    expectations.checks.push(Check::MainReturnNumber {
                        value: (value != "none").then(|| value.to_owned()),
                    })
                }
                ("return-variable", name) if !name.is_empty() => {
                    expectations.checks.push(Check::MainReturnVariable {
                        name: name.to_owned(),
                    })
                }
                ("entry-return", value) if !value.is_empty() => {
                    expectations.checks.push(Check::EntryLastReturnNumber {
                        value: value.to_owned(),
                    })
                }
                ("entry-return-variable", name) if !name.is_empty() => {
                    expectations.checks.push(Check::EntryLastReturnVariable {
                        name: name.to_owned(),
                    })
                }
                ("entry", "") => expectations.checks.push(Check::EntryExists),
                ("no-closure-dispatch", "") => expectations.checks.push(Check::NoClosureDispatch),
                _ => panic!("line {line}: unknown specialization expectation {directive}"),
            }
        }
        if !found {
            return None;
        }
        expectations.loader = loader.expect("specialization source must declare its loader");
        Some(expectations)
    }

    fn specialize(
        &self,
        source_path: &Path,
        source: &str,
    ) -> (&'static TestDb, SourceFile, SpecializeOutput<'static>) {
        let options = SpecializeOptions {
            max_depth: self
                .max_depth
                .unwrap_or(SpecializeOptions::default().max_depth),
            ..SpecializeOptions::default()
        };
        match self.loader {
            Loader::Fixture => {
                assert!(
                    self.max_depth.is_none(),
                    "fixture loading uses default options"
                );
                specialize_fixture_with_db(source_path)
            }
            Loader::Bare => {
                let db = Box::leak(Box::new(TestDb::default()));
                let file = source_file(db, "test", source);
                let module = parse_file_to_hir(db, file).module(db);
                let output = specialize_module(db, module, options);
                (&*db, file, output)
            }
            Loader::Std => specialize_src_with_std_and_db_options(source, options),
        }
    }

    fn assert_matches(
        &self,
        path: &Path,
        db: &TestDb,
        file: SourceFile,
        output: &SpecializeOutput<'_>,
    ) {
        if let Some(clean) = self.frontend_clean {
            let module =
                nameres::module_id_for_source_file(db, file).expect("registered source module");
            let diagnostics = hir_ty::collect_frontend_diagnostics(db, module);
            assert_eq!(
                diagnostics.is_empty(),
                clean,
                "{}: {diagnostics:#?}",
                path.display()
            );
        }
        if self.diagnostics.is_empty() {
            assert!(
                output.diagnostics.is_empty(),
                "{}: {:?}",
                path.display(),
                output.diagnostics
            );
        } else {
            for expected in &self.diagnostics {
                assert!(
                    output.diagnostics.iter().any(|diagnostic| {
                        if !expected.kind.matches(&diagnostic.kind) {
                            return false;
                        }
                        let context = match &diagnostic.kind {
                            SpecializeDiagnosticKind::ComptimeEvaluationFailed { context }
                            | SpecializeDiagnosticKind::MissingEvidence { context } => {
                                Some(context.as_str())
                            }
                            _ => None,
                        };
                        expected
                            .context
                            .as_deref()
                            .is_none_or(|expected| context == Some(expected))
                            && expected.context_contains.as_deref().is_none_or(|expected| {
                                context.is_some_and(|actual| actual.contains(expected))
                            })
                    }),
                    "{}: missing expected diagnostic; actual: {:?}",
                    path.display(),
                    output.diagnostics
                );
            }
        }
        for forbidden in &self.forbidden_diagnostics {
            assert!(
                !output
                    .diagnostics
                    .iter()
                    .any(|diagnostic| forbidden.matches(&diagnostic.kind)),
                "{}: {:?}",
                path.display(),
                output.diagnostics
            );
        }
        for check in &self.checks {
            check.assert_matches(output);
        }
    }
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/audit", glob: "**/main.sol")]
fn audit(fixture: Fixture<&str>) {
    specialization_fixture(fixture);
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/audit", glob: "**/main.invalid.sol.txt")]
fn invalid_audit(fixture: Fixture<&str>) {
    specialization_fixture(fixture);
}

fn specialization_fixture(fixture: Fixture<&str>) {
    let path = PathBuf::from(fixture.path())
        .canonicalize()
        .expect("source fixture path");
    check_source(path, |_, _| {});
}

fn annotated_sources(directory: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    for entry in fs::read_dir(directory).expect("fixture directory") {
        let path = entry.expect("fixture entry").path();
        if path.is_dir() {
            sources.extend(annotated_sources(&path));
        } else if path.extension().is_some_and(|extension| extension == "sol") {
            let source = fs::read_to_string(&path).expect("source fixture");
            if solcore_test_utils::source_directives(&source, "specialize")
                .next()
                .is_some()
            {
                sources.push(path);
            }
        }
    }
    sources.sort();
    sources
}

#[test]
fn multi_module_entries() {
    let fixtures = repo_root().join("crates/specialize/tests/fixtures");
    let audit = fixtures.join("audit");
    let sources = annotated_sources(&fixtures)
        .into_iter()
        .filter(|path| !path.starts_with(&audit))
        .collect::<Vec<_>>();
    assert!(
        !sources.is_empty(),
        "missing multi-module specialization entries"
    );
    for path in sources {
        check_source(path, |_, _| {});
    }
}

#[test]
fn shared_e2e_entries() {
    let sources = annotated_sources(&repo_root().join("tests/e2e"));
    assert!(!sources.is_empty(), "missing shared specialization entries");
    for path in sources {
        check_source(path, |_, _| {});
    }
}

fn check_source(
    path: PathBuf,
    assertion: impl FnOnce(&TestDb, &SpecializeOutput<'_>) + Send + 'static,
) {
    solcore_test_utils::run_in_large_stack(move || {
        let source = fs::read_to_string(&path).expect("source fixture");
        let expectations = Expectations::from_source(&source).expect("specialization expectations");
        let (db, file, output) = expectations.specialize(&path, &source);
        expectations.assert_matches(&path, db, file, &output);
        assertion(db, &output);
    });
}

#[path = "audit_ir.rs"]
mod audit_ir;

fn functions<'a, 'db>(
    output: &'a SpecializeOutput<'db>,
) -> Vec<&'a solcore_specialize::MonoFunction<'db>> {
    output
        .module
        .items
        .iter()
        .filter_map(|item| match item {
            MonoItem::Function(function) => Some(function),
            _ => None,
        })
        .collect()
}

fn entry_function<'a, 'db>(
    output: &'a SpecializeOutput<'db>,
) -> &'a solcore_specialize::MonoFunction<'db> {
    let name = output
        .module
        .items
        .iter()
        .find_map(|item| match item {
            MonoItem::Contract(contract) => contract.entries.iter().find_map(|entry| match entry {
                MonoEntry::RuntimeMain {
                    specialized,
                    origin: MonoRuntimeMainOrigin::User,
                    ..
                } => Some(specialized),
                _ => None,
            }),
            _ => None,
        })
        .or_else(|| output.module.entry_points.first())
        .expect("main entry");
    functions(output)
        .into_iter()
        .find(|function| &function.name == name)
        .expect("entry function")
}

fn last_return<'a, 'db>(function: &'a solcore_specialize::MonoFunction<'db>) -> &'a MonoExpr<'db> {
    match function.body.last().map(|stmt| &stmt.kind) {
        Some(MonoStmtKind::Return(Some(expr))) => expr,
        _ => panic!("missing last return: {function:#?}"),
    }
}

fn statements<'a, 'db>(body: &'a [MonoStmt<'db>], result: &mut Vec<&'a MonoStmt<'db>>) {
    for stmt in body {
        result.push(stmt);
        match &stmt.kind {
            MonoStmtKind::Block(body) => statements(body, result),
            MonoStmtKind::Match { arms, .. } => {
                for arm in arms {
                    statements(&arm.body, result);
                }
            }
            MonoStmtKind::If {
                then_body,
                else_body,
                ..
            } => {
                statements(then_body, result);
                if let Some(body) = else_body {
                    statements(body, result);
                }
            }
            MonoStmtKind::For {
                init, post, body, ..
            } => {
                statements(init, result);
                statements(post, result);
                statements(body, result);
            }
            _ => {}
        }
    }
}

impl Check {
    fn assert_matches(&self, output: &SpecializeOutput<'_>) {
        let all = functions(output);
        match self {
            Self::MainReturnNumber { value } => {
                assert_eq!(&main_return_number(output), value, "{:#?}", output.module)
            }
            Self::MainReturnVariable { name } => assert!(
                matches!(main_return_expr(output).map(|expr| &expr.kind), Some(MonoExprKind::Var(id)) if &id.name == name)
            ),
            Self::EntryLastReturnNumber { value } => assert!(
                matches!(&last_return(entry_function(output)).kind, MonoExprKind::Lit(hir::ast::function::LitKind::Number(number)) if number == value)
            ),
            Self::EntryLastReturnVariable { name } => assert!(
                matches!(&last_return(entry_function(output)).kind, MonoExprKind::Var(id) if &id.name == name)
            ),
            Self::EntryExists => assert!(!output.module.entry_points.is_empty()),
            Self::NoClosureDispatch => assert!(
                all.iter()
                    .all(|function| !function.body.iter().any(stmt_has_closure_dispatch)),
                "{:#?}",
                output.module
            ),
        }
    }
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/fixtures/audit/runtime_string_match_is_rejected_before_emission", glob: "*.sol")]
fn runtime_string_match_is_rejected_before_emission(fixture: Fixture<&str>) {
    let (_, output) = specialize_src(fixture.content());
    assert!(
        output.diagnostics.iter().any(|diagnostic| matches!(
            &diagnostic.kind,
            SpecializeDiagnosticKind::IntegerErasure { context, ty }
                if context == "pattern" && ty == "string"
        )),
        "{:?}",
        output.diagnostics
    );
}
