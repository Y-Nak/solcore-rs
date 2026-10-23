//! Decompose products that contain static callable identities before matching.
//! Their runtime fields remain ordinary match columns; only their known shape
//! disappears. This keeps literal tests and effects in runtime payloads intact.

use super::*;
use crate::evaluate::known::constructor_names_match;

impl<'db> Evaluator<'db> {
    fn callable_product<'a>(&self, value: &'a MonoExpr<'db>) -> Option<&'a MonoExpr<'db>> {
        let value = match &value.kind {
            MonoExprKind::TypeAnnot { expr, .. } => return self.callable_product(expr),
            _ => value,
        };
        (self.value_contains_callable(value)
            && !self.expr_is_known_value(value)
            && matches!(
                value.kind,
                MonoExprKind::Con { .. } | MonoExprKind::Tuple(_)
            ))
        .then_some(value)
    }

    fn callable_columns(&self, value: &MonoExpr<'db>, columns: &mut Vec<MonoExpr<'db>>) {
        match self.callable_product(value).map(|value| &value.kind) {
            Some(MonoExprKind::Con { args, .. } | MonoExprKind::Tuple(args)) => {
                for arg in args {
                    self.callable_columns(arg, columns);
                }
            }
            _ => columns.push(value.clone()),
        }
    }

    pub(super) fn expand_callable_match(
        &self,
        scrutinees: Vec<MonoExpr<'db>>,
        arms: Vec<MonoArm<'db>>,
    ) -> (Vec<MonoExpr<'db>>, Vec<MonoArm<'db>>) {
        if !scrutinees
            .iter()
            .any(|value| self.callable_product(value).is_some())
        {
            return (scrutinees, arms);
        }
        let mut columns = Vec::new();
        for value in &scrutinees {
            self.callable_columns(value, &mut columns);
        }
        let arms = arms
            .into_iter()
            .filter_map(|mut arm| {
                let visible = pattern_bindings(&arm.pats)
                    .into_iter()
                    .collect::<FxHashSet<_>>();
                let mut pats = Vec::new();
                let mut bindings = Vec::new();
                for (pat, value) in arm.pats.iter().zip(&scrutinees) {
                    self.expand_callable_pattern(pat, value, &mut pats, &mut bindings)?;
                }
                // The source pattern's final declaration wins even if an earlier
                // aggregate binder became a synthetic let during decomposition.
                bindings.retain(|(id, _)| visible.contains(id));
                let mut body = bindings
                    .into_iter()
                    .map(|(id, value)| MonoStmt {
                        span: id.span,
                        kind: MonoStmtKind::Let {
                            mode: crate::ir::LetMode::Runtime,
                            ty: Some(id.ty),
                            id,
                            init: Some(value),
                        },
                    })
                    .collect::<Vec<_>>();
                body.append(&mut arm.body);
                arm.body = body;
                arm.pats = pats;
                Some(arm)
            })
            .collect();
        (columns, arms)
    }

    fn expand_callable_pattern(
        &self,
        pat: &MonoPat<'db>,
        value: &MonoExpr<'db>,
        pats: &mut Vec<MonoPat<'db>>,
        bindings: &mut Vec<(MonoId<'db>, MonoExpr<'db>)>,
    ) -> Option<()> {
        let Some(product) = self.callable_product(value) else {
            pats.push(pat.clone());
            return Some(());
        };
        match (&pat.kind, &product.kind) {
            (
                MonoPatKind::Con { ctor, args },
                MonoExprKind::Con {
                    ctor: value_ctor,
                    args: values,
                },
            ) if constructor_names_match(&ctor.name, &value_ctor.name)
                && args.len() == values.len() =>
            {
                for (pat, value) in args.iter().zip(values) {
                    self.expand_callable_pattern(pat, value, pats, bindings)?;
                }
            }
            (MonoPatKind::Tuple(args), MonoExprKind::Tuple(values))
                if args.len() == values.len() =>
            {
                for (pat, value) in args.iter().zip(values) {
                    self.expand_callable_pattern(pat, value, pats, bindings)?;
                }
            }
            (MonoPatKind::Wildcard | MonoPatKind::Var(_), _) => {
                if let MonoPatKind::Var(id) = &pat.kind {
                    bindings.push((id.clone(), value.clone()));
                }
                let mut values = Vec::new();
                self.callable_columns(value, &mut values);
                pats.extend(values.into_iter().map(|value| MonoPat {
                    span: pat.span,
                    ty: value.ty,
                    kind: MonoPatKind::Wildcard,
                }));
            }
            // The constructor is known even when its fields are runtime.
            _ => return None,
        }
        Some(())
    }
}
