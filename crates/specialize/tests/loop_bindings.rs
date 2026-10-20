use std::{collections::BTreeMap, path::PathBuf};

use hir::ast::function::LitKind;
use nameres::{Db as _, ModuleTree, module_id_from_key};
use parser::parse_file_to_hir;
use solcore_specialize::{
    MonoEntry, MonoExpr, MonoExprKind, MonoItem, MonoRuntimeMainOrigin, MonoStmtKind,
    SpecializeDiagnosticKind, SpecializeOptions, specialize_module,
};
use solcore_test_utils::{
    FrontendTestDb as _, define_frontend_test_db, load_main_source,
    load_reachable_modules_with_file_urls, module_fs_snapshot_for_roots,
};

define_frontend_test_db!(TestDb, hir_ty);

fn assert_main_returns_seven(source: &str) {
    inspect_main_return(source, |_, expr| {
        assert!(
            matches!(&expr.kind, MonoExprKind::Lit(LitKind::Number(n)) if n == "7"),
            "static binding was lost: {expr:?}"
        );
    });
}

fn inspect_main_return(source: &str, inspect: impl FnOnce(&TestDb, &MonoExpr<'_>)) {
    let mut db = TestDb::default();
    let key = load_main_source(&mut db, source);
    let std_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std");
    let tree = ModuleTree::new(
        &db,
        PathBuf::from("/main"),
        std_root.clone(),
        BTreeMap::new(),
    );
    db.set_module_tree(tree);
    let snapshot = module_fs_snapshot_for_roots(&db, [std_root.as_path()]);
    db.set_module_fs_snapshot(snapshot);
    load_reachable_modules_with_file_urls(&mut db, key.clone());
    let entry = module_id_from_key(&db, &key);
    let file = db.module_file(entry).unwrap();
    let module = parse_file_to_hir(&db, file).module(&db);
    let diagnostics = hir_ty::collect_frontend_diagnostics(&db, entry);
    assert!(diagnostics.is_empty(), "{source}\n{diagnostics:?}");
    let output = specialize_module(&db, module, SpecializeOptions::default());
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let main = output
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
    let function = output
        .module
        .items
        .iter()
        .find_map(|item| match item {
            MonoItem::Function(function) if &function.name == main => Some(function),
            _ => None,
        })
        .unwrap();
    let Some(MonoStmtKind::Return(Some(expr))) = function.body.last().map(|stmt| &stmt.kind) else {
        panic!("missing return: {function:?}");
    };
    inspect(&db, expr);
}

#[test]
fn loops_preserve_unmodified_comptime_values_and_callables() {
    for binding in [
        "let seed: comptime<word> = 7;",
        "let get: function() returns(word) = lam () -> word { return 7; };",
    ] {
        for body in [
            "for (; false; ) {}",
            "for (; flag; ) { break; }",
            "for (; flag; ) { assembly { mstore(0, 99) } break; }",
            "for (; flag; ) { { let seed: word = 8; seed = 9; } break; }",
            "for (; flag; ) { match (0) { case seed { assembly { seed := 1 } } } break; }",
        ] {
            let result = if binding.starts_with("let seed") {
                "seed"
            } else {
                "get()"
            };
            assert_main_returns_seven(&format!(
                "function main(flag: bool) returns(word) {{ {binding} {body} return {result}; }}"
            ));
        }
    }
}

#[test]
fn unknown_callees_preserve_caller_local_static_values() {
    assert_main_returns_seven(
        "function touch() { assembly { mstore(0, calldatasize()) } }
         function main(flag: bool) returns(word) {
             let seed: comptime<word> = 7;
             for (; flag; ) { touch(); let observed: comptime<word> = seed; break; }
             touch();
             return seed;
         }",
    );
}

#[test]
fn shared_state_effects_do_not_invalidate_same_named_static_locals() {
    for body in [
        "touch();",
        "overwrite();",
        "assembly { mstore(0, calldatasize()) }",
        "for (; flag; ) { touch(); break; }",
        "for (; flag; ) { overwrite(); break; }",
        "for (; flag; ) { assembly { mstore(0, calldatasize()) } break; }",
        "for (; opaque(flag); ) { break; }",
        "{ let other: word = 1; touch(); }",
    ] {
        assert_main_returns_seven(&format!(
            "import * from std;
             import * from std.dispatch;
             function touch() {{ assembly {{ mstore(0, calldatasize()) }} }}
             function opaque(flag: bool) returns(bool) {{ touch(); return flag; }}
             contract C {{
                 seed: word;
                 function overwrite() {{ seed = 9; }}
                 function main() returns(word) {{
                     let flag = opaque(false);
                     let seed: comptime<word> = 7;
                     {body}
                     return seed;
                 }}
             }}"
        ));
    }
}

#[test]
fn residual_calls_use_materialized_argument_types() {
    for intrinsic in ["primAddWord", "addWord", "subWord", "opaqueAdd"] {
        for args in ["input, seed", "seed, input"] {
            inspect_main_return(
                &format!(
                    "import * from std;
                     function opaqueAdd(left: word, right: word) returns(word) {{
                         assembly {{ mstore(0, calldatasize()) left := add(left, right) }}
                         return left;
                     }}
                     function main(input: word) returns(word) {{
                         let seed: comptime<word> = 7;
                         for (; false; ) {{}}
                         return {intrinsic}({args});
                     }}"
                ),
                |db, expr| {
                    let MonoExprKind::Call { callee, args, .. } = &expr.kind else {
                        panic!("expected a residual intrinsic: {expr:?}");
                    };
                    let hir_ty::TyKind::Function { params, ret } = callee.ty.ty().kind(db) else {
                        panic!("expected a function signature: {callee:?}");
                    };
                    assert_eq!(params.len(), args.len());
                    for (param, arg) in params.iter().zip(args) {
                        assert_eq!(*param, arg.ty.ty());
                    }
                    assert_eq!(*ret, expr.ty.ty());
                    assert!(args.iter().any(|arg| {
                        matches!(&arg.kind, MonoExprKind::Lit(LitKind::Number(n)) if n == "7")
                    }));
                },
            );
        }
    }
}

#[test]
fn call_materialization_keeps_unknown_comptime_obligations() {
    for source in [
        "function main(input: word) returns(word) {
             let seed: comptime<word> = input;
             return primAddWord(input, seed);
         }",
        "function restricted(value: comptime<word>) returns(word) {
             assembly { mstore(0, calldatasize()) }
             return value;
         }
         function main(input: word) returns(word) { return restricted(input); }",
    ] {
        let mut db = TestDb::default();
        let key = load_main_source(&mut db, source);
        let entry = module_id_from_key(&db, &key);
        let file = db.module_file(entry).unwrap();
        let module = parse_file_to_hir(&db, file).module(&db);
        let diagnostics = hir_ty::collect_frontend_diagnostics(&db, entry);
        assert!(!diagnostics.is_empty(), "{source}");
        let output = specialize_module(&db, module, SpecializeOptions::default());
        assert!(
            output.diagnostics.iter().any(|diagnostic| matches!(
                diagnostic.kind,
                SpecializeDiagnosticKind::ComptimeEvaluationFailed { .. }
            )),
            "unknown comptime value must remain invalid: {:?}",
            output.diagnostics
        );
    }
}

#[test]
fn residual_match_assignments_preserve_shadowed_static_bindings() {
    for body in [
        "match(flag) { case(seed) { seed = true; } }",
        "match(input) { case(seed) { assembly { seed := 1 } } }",
        "match(input) { case(seed) { seed += 1; } }",
        "match(flag) { case(seed) { if (seed) { seed = false; } } }",
        "match(flag) { case(seed) { { seed = true; } } }",
        "match(flag) { case(seed) { for (; seed; ) { seed = false; } } }",
        "match(flag) { case(seed) { match(seed) { case(inner) { seed = inner; } } } }",
        "match(Box(flag)) { case(Box(seed)) { seed = true; } }",
        "match((flag, flag)) { case((_, seed)) { seed = true; } }",
    ] {
        assert_main_returns_seven(&format!(
            "import * from std;
             enum Box {{ Box(bool) }}
             function main(flag: bool, input: word) returns(word) {{
                 let seed: comptime<word> = 7;
                 {body}
                 return seed;
             }}"
        ));
    }
}

#[test]
fn residual_match_still_invalidates_assignments_to_outer_bindings() {
    inspect_main_return(
        "function main(flag: bool) returns(word) {
             let seed: word = 7;
             match(flag) { case(true) { seed = 9; } case(false) {} }
             return seed;
         }",
        |_, expr| {
            assert!(
                matches!(&expr.kind, MonoExprKind::Var(id) if id.name == "seed"),
                "assignment to outer binding must invalidate its initial value: {expr:?}"
            );
        },
    );
}
