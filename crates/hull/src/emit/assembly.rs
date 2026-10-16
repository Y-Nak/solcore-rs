use super::*;

impl<'db> Emitter<'db> {
    /// Resolve free inline-assembly names through the same lexical bindings as
    /// Core expressions. Control-flow locals can have a different backend name;
    /// raw Yul must not accidentally refer to an older, shadowed source binding.
    pub(super) fn resolve_assembly_bindings(&self, body: &[YulStmt<'db>]) -> Vec<YulStmt<'db>> {
        let mut body = body.to_vec();
        self.resolve_assembly_sequence(&mut body, &mut BTreeSet::new());
        body
    }

    fn resolve_assembly_sequence(&self, body: &mut [YulStmt<'db>], locals: &mut BTreeSet<String>) {
        for stmt in body {
            match &mut stmt.kind {
                YulStmtKind::Let { names, init } => {
                    // The initializer can still read a same-named outer local.
                    if let Some(init) = init {
                        self.resolve_assembly_expr(init, locals);
                    }
                    locals.extend(
                        names
                            .iter()
                            .map(|name| name.atom().text(self.db).to_owned()),
                    );
                }
                YulStmtKind::Assign { names, value } => {
                    for name in names {
                        self.resolve_assembly_name(name, locals);
                    }
                    self.resolve_assembly_expr(value, locals);
                }
                YulStmtKind::Expr(expr) => self.resolve_assembly_expr(expr, locals),
                YulStmtKind::Block(body) => {
                    self.resolve_assembly_sequence(body, &mut locals.clone());
                }
                YulStmtKind::If { cond, body } => {
                    self.resolve_assembly_expr(cond, locals);
                    self.resolve_assembly_sequence(body, &mut locals.clone());
                }
                YulStmtKind::For {
                    init,
                    cond,
                    post,
                    body,
                } => {
                    let mut inner = locals.clone();
                    self.resolve_assembly_sequence(init, &mut inner);
                    self.resolve_assembly_expr(cond, &inner);
                    self.resolve_assembly_sequence(post, &mut inner.clone());
                    self.resolve_assembly_sequence(body, &mut inner);
                }
                YulStmtKind::Switch {
                    expr,
                    cases,
                    default,
                } => {
                    self.resolve_assembly_expr(expr, locals);
                    for case in cases {
                        self.resolve_assembly_sequence(&mut case.body, &mut locals.clone());
                    }
                    if let Some(body) = default {
                        self.resolve_assembly_sequence(body, &mut locals.clone());
                    }
                }
                // Yul functions never capture variables from the surrounding
                // assembly or Core scope. Their names and bodies stay untouched.
                YulStmtKind::FunctionDef { .. }
                | YulStmtKind::Leave
                | YulStmtKind::Break
                | YulStmtKind::Continue
                | YulStmtKind::Error => {}
            }
        }
    }

    fn resolve_assembly_expr(&self, expr: &mut YulExpr<'db>, locals: &BTreeSet<String>) {
        match &mut expr.kind {
            YulExprKind::Ident(name) => self.resolve_assembly_name(name, locals),
            YulExprKind::Call { args, .. } => {
                // Function names inhabit a separate namespace.
                for arg in args {
                    self.resolve_assembly_expr(arg, locals);
                }
            }
            YulExprKind::Lit(_) | YulExprKind::Error => {}
        }
    }

    fn resolve_assembly_name(
        &self,
        name: &mut SpannedElem<'db, Ident<'db>>,
        locals: &BTreeSet<String>,
    ) {
        let source = name.atom().text(self.db);
        if !locals.contains(source)
            && let Some(binding) = self.lookup_expr(source)
            && let ExprKind::Var(backend) = binding.kind
        {
            *name = self.yul_ident(name.span(self.db), backend.as_str());
        }
    }
}
