use dir_test::{Fixture, dir_test};

use super::*;

#[derive(Default)]
struct Case {
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
    StdCurrent,
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

#[derive(Default)]
struct Selector {
    prefix: Option<String>,
    contains: Vec<String>,
    instance_method: bool,
    method: Option<String>,
    entry: bool,
}

impl Selector {
    fn matches(
        &self,
        function: &solcore_specialize::MonoFunction<'_>,
        output: &SpecializeOutput<'_>,
    ) -> bool {
        self.prefix
            .as_ref()
            .is_none_or(|prefix| function.name.starts_with(prefix))
            && self
                .contains
                .iter()
                .all(|part| function.name.contains(part))
            && (!self.entry || output.module.entry_points.contains(&function.name))
            && (!self.instance_method
                || matches!(function.origin, MonoFunctionOrigin::InstanceMethod { .. }))
            && self.method.as_ref().is_none_or(|expected| {
                matches!(
                    &function.origin,
                    MonoFunctionOrigin::InstanceMethod { method, .. } if method == expected
                )
            })
    }
}

enum Check {
    MainReturnNumber {
        value: Option<String>,
    },
    MainReturnVariable {
        name: String,
    },
    EntryLastReturnNumber {
        value: String,
    },
    EntryLastReturnVariable {
        name: String,
    },
    EntryExists,
    NoClosureDispatch,
    UniqueFunctionNames,
    FunctionCount {
        selector: Selector,
        count: Option<usize>,
        min: Option<usize>,
        max: Option<usize>,
    },
    FunctionReturns {
        selector: Selector,
        value: String,
        follow_calls: bool,
        all_paths: bool,
    },
    FunctionParameters {
        selector: Selector,
        names: Vec<String>,
    },
    EqualParameterTypes {
        selector: Selector,
    },
    DistinctReturnTypes {
        selector: Selector,
    },
    Binding {
        name: String,
        count: Option<usize>,
        min: Option<usize>,
        builtin: Option<String>,
    },
    NoTupleBinding {
        name: String,
    },
    ResidualMatch {
        scrutinees: usize,
        arms: usize,
        patterns: usize,
    },
    LoopConditionArgument {
        name: String,
    },
    MaterializedReturnCall {
        literal_argument: String,
    },
}

#[path = "audit_cases.rs"]
mod audit_cases;

macro_rules! source_cases {
    ($name:ident, $directory:literal, $glob:literal) => {
        #[dir_test(dir: $directory, glob: $glob)]
        fn $name(fixture: Fixture<&str>) {
            specialization_fixture(fixture);
        }
    };
}

source_cases!(
    audit,
    "$CARGO_MANIFEST_DIR/tests/fixtures/audit",
    "**/main.sol"
);
source_cases!(
    invalid_audit,
    "$CARGO_MANIFEST_DIR/tests/fixtures/audit",
    "**/main.invalid.sol.txt"
);
source_cases!(
    shared_aggregate_closure_captures,
    "$CARGO_MANIFEST_DIR/../..",
    "tests/e2e/aggregate-closure-captures/main.sol"
);
source_cases!(
    shared_comptime_pattern_scopes,
    "$CARGO_MANIFEST_DIR/../..",
    "tests/e2e/comptime-pattern-scopes/main.sol"
);
source_cases!(
    shared_abi_word_memory,
    "$CARGO_MANIFEST_DIR/../..",
    "tests/e2e/abi-word-memory/main.sol"
);
source_cases!(
    shared_abi_word_custom_codec,
    "$CARGO_MANIFEST_DIR/../..",
    "tests/e2e/abi-word-custom-codec/main.sol"
);
source_cases!(
    shared_abi_byte_memory,
    "$CARGO_MANIFEST_DIR/../..",
    "tests/e2e/abi-byte-memory/main.sol"
);
source_cases!(
    shared_abi_raw_default_codec,
    "$CARGO_MANIFEST_DIR/../..",
    "tests/e2e/abi-raw-default-codec/main.sol"
);
source_cases!(
    function_value_evidence_main,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/function_value_evidence/main.sol"
);
source_cases!(
    module_local_function_evidence_main,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/module_local_function_evidence/main.sol"
);
source_cases!(
    module_local_function_evidence_multiple,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/module_local_function_evidence/multiple.sol"
);
source_cases!(
    module_local_function_evidence_ordered_a,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/module_local_function_evidence/ordered_a.sol"
);
source_cases!(
    module_local_function_evidence_ordered_b,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/module_local_function_evidence/ordered_b.sol"
);
source_cases!(
    module_local_function_evidence_phantom,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/module_local_function_evidence/phantom.sol"
);
source_cases!(
    module_local_function_evidence_values,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/module_local_function_evidence/values.sol"
);
source_cases!(
    operator_evidence_main,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/operator_evidence/main.sol"
);
source_cases!(
    operator_evidence_reversed,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/operator_evidence/reversed.sol"
);
source_cases!(
    superclass_dictionaries_main,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/superclass_dictionaries/main.sol"
);
source_cases!(
    superclass_dictionaries_reversed,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/superclass_dictionaries/reversed.sol"
);
source_cases!(
    trait_method_instantiations_main,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/trait_method_instantiations/main.sol"
);
source_cases!(
    trait_method_instantiations_reversed,
    "$CARGO_MANIFEST_DIR/../..",
    "crates/specialize/tests/fixtures/trait_method_instantiations/reversed.sol"
);

fn specialization_fixture(fixture: Fixture<&str>) {
    let path = PathBuf::from(fixture.path())
        .canonicalize()
        .expect("source fixture path");
    solcore_test_utils::run_in_large_stack(move || {
        let key = solcore_test_utils::fixture_path_key(env!("CARGO_MANIFEST_DIR"), &path);
        let case = audit_cases::expectations(&key);
        let source_path = path.clone();
        let options = SpecializeOptions {
            max_depth: case
                .max_depth
                .unwrap_or(SpecializeOptions::default().max_depth),
            ..SpecializeOptions::default()
        };
        let (db, file, output) = match case.loader {
            Loader::Fixture => {
                assert!(
                    case.max_depth.is_none(),
                    "fixture loading uses default options"
                );
                specialize_fixture_with_db(&source_path)
            }
            Loader::Bare => {
                let source = fs::read_to_string(&source_path).unwrap();
                let db = Box::leak(Box::new(TestDb::default()));
                let file = source_file(db, "test", &source);
                let module = parse_file_to_hir(db, file).module(db);
                let output = specialize_module(db, module, options);
                (&*db, file, output)
            }
            Loader::Std => specialize_src_with_std_and_db_options(
                &fs::read_to_string(&source_path).unwrap(),
                options,
            ),
            Loader::StdCurrent => {
                specialize_with_current_std(&fs::read_to_string(&source_path).unwrap(), options)
            }
        };
        if let Some(clean) = case.frontend_clean {
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
        if case.diagnostics.is_empty() {
            assert!(
                output.diagnostics.is_empty(),
                "{}: {:?}",
                path.display(),
                output.diagnostics
            );
        } else {
            for expected in &case.diagnostics {
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
        for forbidden in &case.forbidden_diagnostics {
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
        for check in &case.checks {
            check.assert_matches(db, &output);
        }
    });
}

fn specialize_with_current_std(
    source: &str,
    options: SpecializeOptions,
) -> (&'static TestDb, SourceFile, SpecializeOutput<'static>) {
    let db = Box::leak(Box::new(TestDb::default()));
    let root = PathBuf::from("/main");
    let std_root = repo_root().join("std");
    db.module_tree = Some(ModuleTree::new(
        db,
        root.clone(),
        std_root.clone(),
        BTreeMap::new(),
    ));
    db.module_fs_snapshot = Some(module_fs_snapshot_for_roots(db, [std_root.as_path()]));
    let path = root.join("main.sol");
    let file = source_file_at_path(db, &path, source);
    let key = module_key_for_path(LibraryId::Main, &root, &path).unwrap();
    db.insert_module_file(key.clone(), file);
    assert!(load_reachable_modules(db, key).is_empty());
    let output = specialize_module(db, parse_file_to_hir(db, file).module(db), options);
    (db, file, output)
}

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

fn selected<'a, 'db>(
    output: &'a SpecializeOutput<'db>,
    selector: &Selector,
) -> Vec<&'a solcore_specialize::MonoFunction<'db>> {
    functions(output)
        .into_iter()
        .filter(|function| selector.matches(function, output))
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
    fn assert_matches(&self, db: &TestDb, output: &SpecializeOutput<'_>) {
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
            Self::UniqueFunctionNames => assert_eq!(
                all.iter()
                    .map(|function| &function.name)
                    .collect::<BTreeSet<_>>()
                    .len(),
                all.len()
            ),
            Self::FunctionCount {
                selector,
                count,
                min,
                max,
            } => {
                let actual = selected(output, selector).len();
                if let Some(count) = count {
                    assert_eq!(actual, *count);
                }
                if let Some(min) = min {
                    assert!(actual >= *min, "{actual} < {min}");
                }
                if let Some(max) = max {
                    assert!(actual <= *max, "{actual} > {max}");
                }
            }
            Self::FunctionReturns {
                selector,
                value,
                follow_calls,
                all_paths,
            } => {
                let matches = selected(output, selector);
                assert!(!matches.is_empty(), "no matching function");
                for mut function in matches {
                    let mut visited = BTreeSet::new();
                    loop {
                        assert!(
                            visited.insert(&function.name),
                            "cyclic return in {}",
                            function.name
                        );
                        let numbers = if *all_paths {
                            function_return_numbers(output, &function.name)
                        } else {
                            match &last_return(function).kind {
                                MonoExprKind::Lit(hir::ast::function::LitKind::Number(number)) => {
                                    vec![number.clone()]
                                }
                                _ => vec![],
                            }
                        };
                        if !numbers.is_empty() {
                            assert!(
                                numbers.iter().all(|number| number == value),
                                "{}: {numbers:?}",
                                function.name
                            );
                            break;
                        }
                        assert!(
                            *follow_calls,
                            "expected literal return in {}",
                            function.name
                        );
                        let MonoExprKind::Call { callee, .. } = &last_return(function).kind else {
                            panic!("unexpected return in {function:#?}");
                        };
                        function = all
                            .iter()
                            .copied()
                            .find(|function| function.name == callee.name)
                            .expect("called function");
                    }
                }
            }
            Self::FunctionParameters { selector, names } => {
                let matches = selected(output, selector);
                assert!(!matches.is_empty());
                for function in matches {
                    assert_eq!(
                        &function
                            .params
                            .iter()
                            .map(|param| param.name.clone())
                            .collect::<Vec<_>>(),
                        names
                    );
                }
            }
            Self::EqualParameterTypes { selector } => {
                let matches = selected(output, selector);
                assert!(matches.len() >= 2);
                for pair in matches.windows(2) {
                    assert_eq!(
                        pair[0].params.iter().map(|p| p.ty).collect::<Vec<_>>(),
                        pair[1].params.iter().map(|p| p.ty).collect::<Vec<_>>()
                    );
                }
            }
            Self::DistinctReturnTypes { selector } => {
                let matches = selected(output, selector);
                assert!(matches.len() >= 2);
                for (index, function) in matches.iter().enumerate() {
                    assert!(
                        matches[index + 1..]
                            .iter()
                            .all(|other| function.ret != other.ret)
                    );
                }
            }
            Self::Binding {
                name,
                count,
                min,
                builtin,
            } => {
                let mut stmts = vec![];
                for function in &all {
                    statements(&function.body, &mut stmts);
                }
                let ids = stmts
                    .iter()
                    .filter_map(|stmt| match &stmt.kind {
                        MonoStmtKind::Let { id, .. } if &id.name == name => Some(id),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if let Some(count) = count {
                    assert_eq!(ids.len(), *count);
                }
                if let Some(min) = min {
                    assert!(ids.len() >= *min);
                }
                if let Some(builtin) = builtin {
                    let ty = match builtin.as_str() {
                        "bool" => BuiltinTyCtor::Bool,
                        "word" => BuiltinTyCtor::Word,
                        _ => panic!("unsupported expected builtin {builtin}"),
                    };
                    assert!(ids.iter().all(|id| id.ty.ty() == Ty::builtin(db, ty)));
                }
            }
            Self::NoTupleBinding { name } => {
                let mut stmts = vec![];
                for function in &all {
                    statements(&function.body, &mut stmts);
                }
                assert!(!stmts.iter().any(|stmt| matches!(&stmt.kind, MonoStmtKind::Let { id, init: Some(init), .. } if &id.name == name && matches!(init.kind, MonoExprKind::Tuple(_)))));
            }
            Self::ResidualMatch {
                scrutinees,
                arms,
                patterns,
            } => {
                let function = entry_function(output);
                let (actual_inputs, actual_arms) = function
                    .body
                    .iter()
                    .find_map(|stmt| match &stmt.kind {
                        MonoStmtKind::Match { scrutinees, arms } => Some((scrutinees, arms)),
                        _ => None,
                    })
                    .expect("residual match");
                assert_eq!(actual_inputs.len(), *scrutinees);
                assert_eq!(actual_arms.len(), *arms);
                assert!(actual_arms.iter().all(|arm| arm.pats.len() == *patterns));
            }
            Self::LoopConditionArgument { name } => {
                let condition = all
                    .iter()
                    .find_map(|function| {
                        let mut body = Vec::new();
                        statements(&function.body, &mut body);
                        body.into_iter().find_map(|stmt| match &stmt.kind {
                            MonoStmtKind::For { cond, .. } => Some(cond),
                            _ => None,
                        })
                    })
                    .expect("residual loop");
                assert!(
                    matches!(&condition.kind, MonoExprKind::Call { args, .. } if args.iter().any(|arg| matches!(&arg.kind, MonoExprKind::Var(id) if &id.name == name)))
                );
            }
            Self::MaterializedReturnCall { literal_argument } => {
                let expr = last_return(entry_function(output));
                let MonoExprKind::Call { callee, args, .. } = &expr.kind else {
                    panic!("expected residual call: {expr:?}");
                };
                let TyKind::Function { params, ret } = callee.ty.ty().kind(db) else {
                    panic!("expected function signature");
                };
                assert_eq!(params.len(), args.len());
                for (param, arg) in params.iter().zip(args) {
                    assert_eq!(*param, arg.ty.ty());
                }
                assert_eq!(*ret, expr.ty.ty());
                assert!(args.iter().any(|arg| matches!(&arg.kind, MonoExprKind::Lit(hir::ast::function::LitKind::Number(number)) if number == literal_argument)));
            }
        }
    }
}
