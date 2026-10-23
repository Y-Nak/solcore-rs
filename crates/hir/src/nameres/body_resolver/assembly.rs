use super::*;
use crate::ast::function::{YulExpr, YulExprKind, YulStmt, YulStmtKind};

impl<'db> BodyResolver<'db, '_> {
    pub(super) fn assembly(&mut self, body: &[YulStmt<'db>]) {
        self.assembly_sequence(body, &mut FxHashSet::default());
    }

    fn assembly_sequence(&mut self, body: &[YulStmt<'db>], locals: &mut FxHashSet<String>) {
        for stmt in body {
            match &stmt.kind {
                YulStmtKind::Let { names, init } => {
                    // A declaration's initializer still sees the outer binding.
                    if let Some(init) = init {
                        self.assembly_expr(init, locals);
                    }
                    locals.extend(names.iter().map(|name| ident_text(self.db, name)));
                }
                YulStmtKind::Assign { names, value } => {
                    for name in names {
                        self.assembly_name(name, locals);
                    }
                    self.assembly_expr(value, locals);
                }
                YulStmtKind::Expr(expr) => self.assembly_expr(expr, locals),
                YulStmtKind::Block(body) => {
                    self.assembly_sequence(body, &mut locals.clone());
                }
                YulStmtKind::If { cond, body } => {
                    self.assembly_expr(cond, locals);
                    self.assembly_sequence(body, &mut locals.clone());
                }
                YulStmtKind::For {
                    init,
                    cond,
                    post,
                    body,
                } => {
                    // Init bindings are visible throughout the loop; body and
                    // post bindings belong to separate nested scopes.
                    let mut inner = locals.clone();
                    self.assembly_sequence(init, &mut inner);
                    self.assembly_expr(cond, &inner);
                    self.assembly_sequence(body, &mut inner.clone());
                    self.assembly_sequence(post, &mut inner);
                }
                YulStmtKind::Switch {
                    expr,
                    cases,
                    default,
                } => {
                    self.assembly_expr(expr, locals);
                    for case in cases {
                        self.assembly_sequence(&case.body, &mut locals.clone());
                    }
                    if let Some(body) = default {
                        self.assembly_sequence(body, &mut locals.clone());
                    }
                }
                // Yul functions cannot capture source or surrounding Yul
                // variables. Their parameters, returns and local references
                // therefore never name an enclosing source binding.
                YulStmtKind::FunctionDef { .. }
                | YulStmtKind::Leave
                | YulStmtKind::Break
                | YulStmtKind::Continue
                | YulStmtKind::Error => {}
            }
        }
    }

    fn assembly_expr(&mut self, expr: &YulExpr<'db>, locals: &FxHashSet<String>) {
        match &expr.kind {
            YulExprKind::Ident(name) => self.assembly_name(name, locals),
            YulExprKind::Call { args, .. } => {
                // Calls use Yul's function namespace, never source variables.
                for arg in args {
                    self.assembly_expr(arg, locals);
                }
            }
            YulExprKind::Lit(_) | YulExprKind::Error => {}
        }
    }

    fn assembly_name(&mut self, name: &SpannedElem<'db, Ident<'db>>, locals: &FxHashSet<String>) {
        let text = ident_text_str(self.db, name);
        if !locals.contains(text)
            && let Some(resolution) = self.lookup_local(text)
        {
            self.map.assembly.push(BodyAssemblyResolution {
                span: name.span(self.db),
                resolution,
            });
        }
    }
}
