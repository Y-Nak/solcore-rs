use hir::input::SourceFile;
use hir_ty::{FrontendDesugarPlan, Ty};
use parser::parse_file_to_hir;
use solcore_test_utils::define_frontend_test_db;

use super::*;
use crate::ir::{MonoFunctionOrigin, MonoParam, ParamMode};

define_frontend_test_db!(TestDb, hir_ty);

#[test]
fn expression_match_bindings_inherit_scrutinee_availability_and_shadow_outer_names() {
    let db = TestDb::default();
    let file = SourceFile::new(
        &db,
        "memory:///pattern.sol".parse().unwrap(),
        Some(String::new()),
    );
    let hir = parse_file_to_hir(&db, file).module(&db);
    let span = hir.span(&db);
    let word = MonoTy::new_unchecked(Ty::builtin(&db, BuiltinTyCtor::Word));
    let var = |name: &str| MonoExpr {
        span,
        ty: word,
        kind: MonoExprKind::Var(MonoId {
            span,
            ty: word,
            name: name.to_owned(),
        }),
    };
    let module = MonoModule {
        module: hir.def_id(&db),
        frontend_desugar: FrontendDesugarPlan { bodies: Vec::new() },
        entry_points: vec!["take".to_owned()],
        items: vec![MonoItem::Function(MonoFunction {
            origin: MonoFunctionOrigin::Source,
            source: None,
            shadowed_top_level: None,
            name: "take".to_owned(),
            span,
            params: vec![MonoParam {
                name: "value".to_owned(),
                mode: ParamMode::Comptime,
                ty: word,
                span,
            }],
            ret: word,
            comptime_obligations: Vec::new(),
            body: vec![MonoStmt {
                span,
                kind: MonoStmtKind::Return(Some(var("value"))),
            }],
        })],
    };
    let match_expr = MonoExpr {
        span,
        ty: word,
        kind: MonoExprKind::Match {
            scrutinee: Box::new(var("x")),
            arms: vec![MonoExprArm {
                span,
                pat: MonoPat {
                    span,
                    ty: word,
                    kind: MonoPatKind::Var(MonoId {
                        name: "y".to_owned(),
                        span,
                        ty: word,
                    }),
                },
                expr: MonoExpr {
                    span,
                    ty: word,
                    kind: MonoExprKind::Call {
                        callee: MonoId {
                            span,
                            name: "take".to_owned(),
                            ty: MonoTy::new_unchecked(Ty::function(
                                &db,
                                vec![word.ty()],
                                word.ty(),
                            )),
                        },
                        args: vec![var("y")],
                        origin: MonoCallOrigin::ByName,
                    },
                },
            }],
        },
    };
    let env = VEnv::from_iter([("y".to_owned(), int_expr(BigInt::from_u64(7), word, span))]);
    for (outer_static, expected_static) in [("x", true), ("y", false)] {
        let mut evaluator = Evaluator::new(&db, &module, 100, 32);
        let comptime_env = CEnv::from_iter([outer_static.to_owned()]);
        assert_eq!(
            evaluator.expr_is_comptime(&match_expr, &comptime_env),
            expected_static
        );
        let result = evaluator.eval_expr(&env, &comptime_env, match_expr.clone());
        assert_eq!(
            evaluator.diagnostics.is_empty(),
            expected_static,
            "{:?}",
            evaluator.diagnostics
        );
        let MonoExprKind::Match { arms, .. } = &result.kind else {
            panic!("unknown scrutinee must preserve match: {result:?}");
        };
        let MonoExprKind::Call { args, .. } = &arms[0].expr.kind else {
            panic!("pattern binding must shadow outer constant: {result:?}");
        };
        assert!(matches!(&args[0].kind, MonoExprKind::Var(id) if id.name == "y"));
    }
}
