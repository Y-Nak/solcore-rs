//! Keep known callable identities static while passing captured values as
//! ordinary runtime arguments. Lift lambda bodies and specialize higher-order
//! callees without inlining their statements, so call boundaries and effects
//! survive even when partial evaluation cannot produce a constant result.

use super::*;
use crate::evaluate::{
    effects::{function_is_comptime_pure, function_is_pure},
    erasure::{strip_comptime, ty_is_function},
    known::collect_pat_binders,
};
use crate::ir::{MonoFunctionOrigin, MonoParam, ParamMode, visit::walk_expr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ClosureCloneKey<'db> {
    callee: String,
    // Capture values are runtime arguments, not part of specialization identity.
    bindings: Vec<(usize, StaticCallableKey<'db>)>,
}

/// The static structure retained by a callable specialization. Constructor
/// choices and explicitly static fields matter just as callable identities do;
/// ordinary payloads are excluded so recursive calls reuse a clone.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum StaticCallableKey<'db> {
    Callable(String),
    Constructor(String, Vec<Self>),
    Tuple(Vec<Self>),
    Literal(LitKind),
    Proxy(MonoTy<'db>),
    Runtime(MonoTy<'db>),
}

struct ParameterizedCallable<'db> {
    key: StaticCallableKey<'db>,
    value: MonoExpr<'db>,
    payload: Vec<(MonoId<'db>, MonoExpr<'db>)>,
}

pub(super) struct LiftedClosure<'db> {
    expr: MonoExpr<'db>,
    target: MonoId<'db>,
    captures: Vec<MonoId<'db>>,
}

impl<'db> Evaluator<'db> {
    /// Callable identities have no runtime representation, including when they
    /// occur inside a statically known product or constructor. Such values stay
    /// in the evaluator; only their captured runtime payload is materialized.
    pub(super) fn value_contains_callable(&self, expr: &MonoExpr<'db>) -> bool {
        match &expr.kind {
            MonoExprKind::Lambda { .. } => true,
            MonoExprKind::Var(id) => self.functions.contains_key(&id.name),
            MonoExprKind::Con { args, .. } | MonoExprKind::Tuple(args) => {
                args.iter().any(|arg| self.value_contains_callable(arg))
            }
            MonoExprKind::TypeAnnot { expr, .. } => self.value_contains_callable(expr),
            _ => false,
        }
    }

    pub(super) fn capture_closure_value(
        &mut self,
        expr: MonoExpr<'db>,
    ) -> (MonoExpr<'db>, Vec<MonoStmt<'db>>) {
        let materialize_payload = self.value_contains_callable(&expr);
        self.capture_static_value(expr, materialize_payload)
    }

    fn capture_static_value(
        &mut self,
        expr: MonoExpr<'db>,
        materialize_payload: bool,
    ) -> (MonoExpr<'db>, Vec<MonoStmt<'db>>) {
        let expr = match expr {
            MonoExpr {
                span,
                ty,
                kind: MonoExprKind::Con { ctor, args },
            } => {
                let (args, bindings) = self.capture_closure_values(args, materialize_payload);
                return (
                    MonoExpr {
                        span,
                        ty,
                        kind: MonoExprKind::Con { ctor, args },
                    },
                    bindings,
                );
            }
            MonoExpr {
                span,
                ty,
                kind: MonoExprKind::Tuple(elems),
            } => {
                let (elems, bindings) = self.capture_closure_values(elems, materialize_payload);
                return (
                    MonoExpr {
                        span,
                        ty,
                        kind: MonoExprKind::Tuple(elems),
                    },
                    bindings,
                );
            }
            MonoExpr {
                span,
                ty,
                kind:
                    MonoExprKind::TypeAnnot {
                        expr,
                        ty: annotation,
                    },
            } => {
                let (expr, bindings) = self.capture_static_value(*expr, materialize_payload);
                return (
                    MonoExpr {
                        span,
                        ty,
                        kind: MonoExprKind::TypeAnnot {
                            expr: Box::new(expr),
                            ty: annotation,
                        },
                    },
                    bindings,
                );
            }
            expr => expr,
        };
        let MonoExprKind::Lambda { params, body, .. } = &expr.kind else {
            if materialize_payload && !self.expr_is_known_value(&expr) {
                let (value, binding) = self.materialize_callable_payload(expr);
                return (value, vec![binding]);
            }
            return (expr, Vec::new());
        };
        if self.closure_captures(params, body).is_empty() {
            return (expr, Vec::new());
        }
        let Some((target, captures)) = self.lift_closure(&expr) else {
            return (expr, Vec::new());
        };
        let mut bindings = Vec::new();
        let mut values = Vec::new();
        for capture in captures {
            let (value, binding) = self.materialize_callable_payload(capture);
            values.push(value);
            bindings.push(binding);
        }
        // Capture the value at closure construction, before a later assignment
        // or shadowing declaration can change what its free variable denotes.
        (
            self.bound_closure(&target, values, expr.ty, expr.span),
            bindings,
        )
    }

    fn materialize_callable_payload(
        &mut self,
        value: MonoExpr<'db>,
    ) -> (MonoExpr<'db>, MonoStmt<'db>) {
        let id = MonoId {
            name: self.fresh_clone_name("$captured"),
            ty: value.ty,
            span: value.span,
        };
        (
            var_expr(id.clone()),
            MonoStmt {
                span: value.span,
                kind: MonoStmtKind::Let {
                    mode: crate::ir::LetMode::Runtime,
                    ty: Some(id.ty),
                    id,
                    init: Some(value),
                },
            },
        )
    }

    fn capture_closure_values(
        &mut self,
        values: Vec<MonoExpr<'db>>,
        materialize_payload: bool,
    ) -> (Vec<MonoExpr<'db>>, Vec<MonoStmt<'db>>) {
        let mut bindings = Vec::new();
        let values = values
            .into_iter()
            .map(|value| {
                let (value, mut captures) = self.capture_static_value(value, materialize_payload);
                bindings.append(&mut captures);
                value
            })
            .collect();
        (values, bindings)
    }

    pub(super) fn try_clone_closure_call(
        &mut self,
        callee: &MonoId<'db>,
        args: &[MonoExpr<'db>],
        result_ty: MonoTy<'db>,
        span: Span<'db>,
    ) -> Option<MonoExpr<'db>> {
        let function = self.functions.get(&callee.name)?.clone();
        // std.dispatch already substitutes its known arguments while inlining
        // unit-returning statement bodies. Preserve that established path.
        if self.function_is_std_dispatch(&function) || function.params.len() != args.len() {
            return None;
        }
        let mut used_names = build_type_reg(&function.params, &function.body)
            .into_keys()
            .collect::<FxHashSet<_>>();
        let mut bindings = Vec::new();
        for (index, (param, arg)) in function.params.iter().zip(args).enumerate() {
            if ty_is_function(self.db, param.ty.ty()) || self.value_contains_callable(arg) {
                // Unknown function parameters remain in the unspecialized body.
                // A caller with a concrete closure creates a separate definition.
                let value = self.parameterize_callable_value(
                    arg,
                    index,
                    param_is_comptime(self.db, param),
                    &mut used_names,
                )?;
                bindings.push((index, value));
            }
        }
        if bindings.is_empty() {
            return None;
        }
        let key = ClosureCloneKey {
            callee: callee.name.clone(),
            bindings: bindings
                .iter()
                .map(|(index, value)| (*index, value.key.clone()))
                .collect(),
        };
        let mut params = Vec::new();
        let mut clone_args = Vec::new();
        let mut env = VEnv::default();
        for (index, (param, arg)) in function.params.iter().zip(args).enumerate() {
            let Some((_, value)) = bindings.iter().find(|(i, _)| *i == index) else {
                params.push(param.clone());
                clone_args.push(arg.clone());
                continue;
            };
            for (id, capture) in &value.payload {
                params.push(MonoParam {
                    name: id.name.clone(),
                    mode: ParamMode::Runtime,
                    ty: id.ty,
                    span: id.span,
                });
                clone_args.push(capture.clone());
            }
            env.insert(param.name.clone(), value.value.clone());
        }
        let clone_ty = function_ty(self.db, &params, function.ret);
        let name = if let Some(name) = self.closure_clone_names.get(&key) {
            name.clone()
        } else {
            if self.clone_fuel == 0 {
                self.push_closure_fuel_diagnostic(&function, span);
                return None;
            }
            self.clone_fuel -= 1;
            let name = self.fresh_clone_name(&format!("{}$closure", function.name));
            let mut clone = function;
            clone.name = name.clone();
            clone.params = params;
            // Register before evaluating the body so recursive combinators reuse
            // their specialization instead of creating an unbounded clone chain.
            self.closure_clone_names.insert(key, name.clone());
            self.register_closure_function(clone, env);
            name
        };
        Some(MonoExpr {
            span,
            ty: result_ty,
            kind: MonoExprKind::Call {
                callee: MonoId {
                    name,
                    ty: clone_ty,
                    span: callee.span,
                },
                args: clone_args,
                origin: MonoCallOrigin::ByName,
            },
        })
    }

    /// Keep callable identities and product structure static, passing each
    /// captured value or ordinary runtime field as an explicit clone parameter.
    fn parameterize_callable_value(
        &mut self,
        expr: &MonoExpr<'db>,
        index: usize,
        static_payload: bool,
        used_names: &mut FxHashSet<String>,
    ) -> Option<ParameterizedCallable<'db>> {
        let mut payload = Vec::new();
        let (key, value) = self.parameterize_callable_value_inner(
            expr,
            index,
            static_payload,
            used_names,
            &mut payload,
        )?;
        Some(ParameterizedCallable {
            key,
            value,
            payload,
        })
    }

    fn parameterize_callable_value_inner(
        &mut self,
        expr: &MonoExpr<'db>,
        index: usize,
        static_payload: bool,
        used_names: &mut FxHashSet<String>,
        payload: &mut Vec<(MonoId<'db>, MonoExpr<'db>)>,
    ) -> Option<(StaticCallableKey<'db>, MonoExpr<'db>)> {
        let static_payload = static_payload
            || ty_is_comptime(self.db, expr.ty.ty())
            || ty_is_comptime_string(self.db, expr.ty.ty())
            || ty_is_builtin(self.db, expr.ty.ty(), BuiltinTyCtor::Integer);
        if !static_payload
            && !self.value_contains_callable(expr)
            && !matches!(&expr.kind, MonoExprKind::Proxy(_))
        {
            if ty_is_function(self.db, expr.ty.ty()) {
                return None;
            }
            return Some((
                StaticCallableKey::Runtime(expr.ty),
                callable_payload_parameter(expr.clone(), index, used_names, payload),
            ));
        }
        let mut value = expr.clone();
        let key = match &expr.kind {
            MonoExprKind::Con { ctor, args } => {
                let (keys, values): (Vec<_>, Vec<_>) = args
                    .iter()
                    .map(|arg| {
                        self.parameterize_callable_value_inner(
                            arg,
                            index,
                            static_payload,
                            used_names,
                            payload,
                        )
                    })
                    .collect::<Option<Vec<_>>>()?
                    .into_iter()
                    .unzip();
                value.kind = MonoExprKind::Con {
                    ctor: ctor.clone(),
                    args: values,
                };
                StaticCallableKey::Constructor(ctor.name.clone(), keys)
            }
            MonoExprKind::Tuple(args) => {
                let (keys, values): (Vec<_>, Vec<_>) = args
                    .iter()
                    .map(|arg| {
                        self.parameterize_callable_value_inner(
                            arg,
                            index,
                            static_payload,
                            used_names,
                            payload,
                        )
                    })
                    .collect::<Option<Vec<_>>>()?
                    .into_iter()
                    .unzip();
                value.kind = MonoExprKind::Tuple(values);
                StaticCallableKey::Tuple(keys)
            }
            MonoExprKind::TypeAnnot { expr, ty } => {
                let (key, inner) = self.parameterize_callable_value_inner(
                    expr,
                    index,
                    static_payload,
                    used_names,
                    payload,
                )?;
                value.kind = MonoExprKind::TypeAnnot {
                    expr: Box::new(inner),
                    ty: *ty,
                };
                key
            }
            MonoExprKind::Lit(lit) => StaticCallableKey::Literal(lit.clone()),
            MonoExprKind::Proxy(ty) => StaticCallableKey::Proxy(*ty),
            _ => {
                if let Some((target, bound)) = self.lift_closure(expr) {
                    let values = bound
                        .into_iter()
                        .map(|capture| {
                            callable_payload_parameter(capture, index, used_names, payload)
                        })
                        .collect();
                    value = self.bound_closure(&target, values, expr.ty, expr.span);
                    StaticCallableKey::Callable(target.name)
                } else if !ty_is_function(self.db, expr.ty.ty()) {
                    value = callable_payload_parameter(expr.clone(), index, used_names, payload);
                    StaticCallableKey::Runtime(expr.ty)
                } else {
                    return None;
                }
            }
        };
        Some((key, value))
    }

    pub(super) fn lift_closure(
        &mut self,
        expr: &MonoExpr<'db>,
    ) -> Option<(MonoId<'db>, Vec<MonoExpr<'db>>)> {
        match &expr.kind {
            MonoExprKind::TypeAnnot { expr, .. } => return self.lift_closure(expr),
            MonoExprKind::Var(id) if self.functions.contains_key(&id.name) => {
                return Some((id.clone(), Vec::new()));
            }
            _ => {}
        }
        let MonoExprKind::Lambda { name, params, body } = &expr.kind else {
            return None;
        };
        // A bound closure is a forwarding lambda built below. Preserve its
        // original target identity when it crosses another function boundary.
        if name.starts_with("$bound_closure:")
            && let [
                MonoStmt {
                    kind:
                        MonoStmtKind::Return(Some(MonoExpr {
                            kind: MonoExprKind::Call { callee, args, .. },
                            ..
                        })),
                    ..
                },
            ] = body.as_slice()
        {
            return Some((callee.clone(), args[params.len()..].to_vec()));
        }
        if let Some(lifted) = self
            .lifted_closures
            .iter()
            .find(|lifted| lifted.expr == *expr)
        {
            return Some((
                lifted.target.clone(),
                lifted.captures.iter().cloned().map(var_expr).collect(),
            ));
        }
        let TyKind::Function { ret, .. } = strip_comptime(self.db, expr.ty.ty()).kind(self.db)
        else {
            return None;
        };
        let ret = MonoTy::new_unchecked(*ret);
        let captures = self.closure_captures(params, body);
        let mut lifted_params = params.clone();
        lifted_params.extend(captures.iter().map(|id| MonoParam {
            name: id.name.clone(),
            mode: ParamMode::Runtime,
            ty: id.ty,
            span: id.span,
        }));
        let function = MonoFunction {
            origin: MonoFunctionOrigin::Source,
            source: None,
            shadowed_top_level: None,
            name: self.fresh_clone_name("__lambda"),
            span: expr.span,
            params: lifted_params,
            ret,
            comptime_obligations: Vec::new(),
            body: body.clone(),
        };
        if self.clone_fuel == 0 {
            self.push_closure_fuel_diagnostic(&function, expr.span);
            return None;
        }
        self.clone_fuel -= 1;
        let target = MonoId {
            name: function.name.clone(),
            ty: function_ty(self.db, &function.params, ret),
            span: expr.span,
        };
        self.register_closure_function(function, VEnv::default());
        self.lifted_closures.push(LiftedClosure {
            expr: expr.clone(),
            target: target.clone(),
            captures: captures.clone(),
        });
        Some((target, captures.into_iter().map(var_expr).collect()))
    }

    fn bound_closure(
        &self,
        target: &MonoId<'db>,
        captures: Vec<MonoExpr<'db>>,
        ty: MonoTy<'db>,
        span: Span<'db>,
    ) -> MonoExpr<'db> {
        if captures.is_empty() {
            return MonoExpr {
                span,
                ty,
                kind: MonoExprKind::Var(MonoId {
                    ty,
                    ..target.clone()
                }),
            };
        }
        let TyKind::Function {
            params: param_tys,
            ret,
        } = strip_comptime(self.db, ty.ty()).kind(self.db)
        else {
            unreachable!("function parameter")
        };
        let params = param_tys
            .iter()
            .enumerate()
            .map(|(index, ty)| MonoParam {
                name: format!("$arg{index}"),
                mode: ParamMode::Runtime,
                ty: MonoTy::new_unchecked(*ty),
                span,
            })
            .collect::<Vec<_>>();
        let mut args = params
            .iter()
            .map(|param| {
                var_expr(MonoId {
                    name: param.name.clone(),
                    ty: param.ty,
                    span,
                })
            })
            .collect::<Vec<_>>();
        args.extend(captures);
        MonoExpr {
            span,
            ty,
            kind: MonoExprKind::Lambda {
                name: format!("$bound_closure:{}", target.name),
                params,
                body: vec![MonoStmt {
                    span,
                    kind: MonoStmtKind::Return(Some(MonoExpr {
                        span,
                        ty: MonoTy::new_unchecked(*ret),
                        kind: MonoExprKind::Call {
                            callee: target.clone(),
                            args,
                            origin: MonoCallOrigin::ByName,
                        },
                    })),
                }],
            },
        }
    }

    fn register_closure_function(&mut self, function: MonoFunction<'db>, env: VEnv<'db>) {
        if function_is_pure(
            self.db,
            &function,
            &self.pure_funs,
            &function.name,
            &self.storage_fields,
        ) {
            self.pure_funs.insert(function.name.clone());
        }
        if function_is_comptime_pure(
            self.db,
            &function,
            &self.comptime_pure_funs,
            &function.name,
            &self.storage_fields,
        ) {
            self.comptime_pure_funs.insert(function.name.clone());
        }
        // A closure's writes depend on its bound callable arguments. Until its
        // body has been evaluated, use a conservative summary at all call sites.
        self.write_effects
            .insert(function.name.clone(), AssignedNames::All);
        self.functions
            .insert(function.name.clone(), function.clone());
        self.pending_clones
            .push_back(PendingClone { function, env });
    }

    fn push_closure_fuel_diagnostic(&mut self, function: &MonoFunction<'db>, span: Span<'db>) {
        self.push_inline_limit_diagnostic(
            display_mono_function_name(self.db, function),
            self.inline_chain_is_comptime(self.comptime_mode),
            span,
            InlineBudgetExhaustion::TotalWork {
                limit: self.fuel_limit,
            },
        );
    }

    pub(super) fn closure_captures(
        &self,
        params: &[MonoParam<'db>],
        body: &[MonoStmt<'db>],
    ) -> Vec<MonoId<'db>> {
        let mut visitor = CaptureCollector {
            evaluator: self,
            locals: params.iter().map(|param| param.name.clone()).collect(),
            captures: BTreeMap::new(),
        };
        visitor.stmts(body);
        visitor.captures.into_values().collect()
    }
}

fn callable_payload_parameter<'db>(
    value: MonoExpr<'db>,
    index: usize,
    used_names: &mut FxHashSet<String>,
    payload: &mut Vec<(MonoId<'db>, MonoExpr<'db>)>,
) -> MonoExpr<'db> {
    // `$` is outside the source identifier alphabet. These parameters cannot
    // capture a wrapper's source locals or lambda parameters.
    let mut name = format!("$capture{index}_{}", payload.len());
    while !used_names.insert(name.clone()) {
        name.push('_');
    }
    let id = MonoId {
        name,
        ty: value.ty,
        span: value.span,
    };
    payload.push((id.clone(), value));
    var_expr(id)
}

fn function_ty<'db>(db: &'db dyn Db, params: &[MonoParam<'db>], ret: MonoTy<'db>) -> MonoTy<'db> {
    MonoTy::new_unchecked(hir_ty::Ty::function(
        db,
        params.iter().map(|param| param.ty.ty()).collect(),
        ret.ty(),
    ))
}

fn var_expr(id: MonoId<'_>) -> MonoExpr<'_> {
    MonoExpr {
        span: id.span,
        ty: id.ty,
        kind: MonoExprKind::Var(id),
    }
}

struct CaptureCollector<'eval, 'db> {
    evaluator: &'eval Evaluator<'db>,
    locals: FxHashSet<String>,
    captures: BTreeMap<String, MonoId<'db>>,
}

impl<'db> CaptureCollector<'_, 'db> {
    fn capture(&mut self, id: &MonoId<'db>) {
        if !self.locals.contains(&id.name)
            && !self.evaluator.functions.contains_key(&id.name)
            && (!self.evaluator.storage_fields.contains(&id.name)
                || self.evaluator.capture_types.contains_key(&id.name))
        {
            self.captures
                .entry(id.name.clone())
                .or_insert_with(|| id.clone());
        }
    }

    fn stmts(&mut self, body: &[MonoStmt<'db>]) {
        let saved = self.locals.clone();
        self.stmts_in_scope(body);
        self.locals = saved;
    }

    fn stmts_in_scope(&mut self, body: &[MonoStmt<'db>]) {
        for stmt in body {
            self.visit_stmt(stmt);
        }
    }

    fn yul_name(&mut self, name: &hir::span::SpannedElem<'db, hir::ast::Ident<'db>>) {
        let name = ident_text(self.evaluator.db, name);
        if let Some(id) = self.evaluator.capture_types.get(&name) {
            self.capture(id);
        }
    }

    fn yul_expr(&mut self, expr: &YulExpr<'db>) {
        match &expr.kind {
            YulExprKind::Ident(name) => self.yul_name(name),
            YulExprKind::Call { args, .. } => {
                for arg in args {
                    self.yul_expr(arg);
                }
            }
            _ => {}
        }
    }

    fn yul_stmts(&mut self, stmts: &[YulStmt<'db>]) {
        let saved = self.locals.clone();
        for stmt in stmts {
            self.yul_stmt(stmt);
        }
        self.locals = saved;
    }

    fn yul_stmt(&mut self, stmt: &YulStmt<'db>) {
        match &stmt.kind {
            YulStmtKind::Let { names, init } => {
                if let Some(init) = init {
                    self.yul_expr(init);
                }
                self.locals
                    .extend(names.iter().map(|name| ident_text(self.evaluator.db, name)));
            }
            YulStmtKind::Assign { names, value } => {
                self.yul_expr(value);
                for name in names {
                    self.yul_name(name);
                }
            }
            YulStmtKind::Expr(expr) => self.yul_expr(expr),
            YulStmtKind::Block(body) => self.yul_stmts(body),
            YulStmtKind::If { cond, body } => {
                self.yul_expr(cond);
                self.yul_stmts(body);
            }
            YulStmtKind::For {
                init,
                cond,
                post,
                body,
            } => {
                let saved = self.locals.clone();
                for stmt in init {
                    self.yul_stmt(stmt);
                }
                self.yul_expr(cond);
                self.yul_stmts(body);
                self.yul_stmts(post);
                self.locals = saved;
            }
            YulStmtKind::Switch {
                expr,
                cases,
                default,
            } => {
                self.yul_expr(expr);
                for case in cases {
                    self.yul_stmts(&case.body);
                }
                if let Some(body) = default {
                    self.yul_stmts(body);
                }
            }
            // Yul functions have their own scope and cannot capture Core locals.
            YulStmtKind::FunctionDef { .. }
            | YulStmtKind::Leave
            | YulStmtKind::Break
            | YulStmtKind::Continue
            | YulStmtKind::Error => {}
        }
    }
}

impl<'db> Visitor<'db> for CaptureCollector<'_, 'db> {
    fn visit_stmt(&mut self, stmt: &MonoStmt<'db>) {
        match &stmt.kind {
            MonoStmtKind::Let { id, init, .. } => {
                if let Some(init) = init {
                    self.visit_expr(init);
                }
                self.locals.insert(id.name.clone());
            }
            MonoStmtKind::Block(body) => self.stmts(body),
            MonoStmtKind::Assembly(body) => self.yul_stmts(body),
            MonoStmtKind::If {
                cond,
                then_body,
                else_body,
            } => {
                self.visit_expr(cond);
                self.stmts_in_scope(then_body);
                if let Some(body) = else_body {
                    self.stmts_in_scope(body);
                }
            }
            MonoStmtKind::Match { scrutinees, arms } => {
                for expr in scrutinees {
                    self.visit_expr(expr);
                }
                for arm in arms {
                    if self.evaluator.if_stmt_spans.contains(&stmt.span) {
                        self.stmts_in_scope(&arm.body);
                        continue;
                    }
                    let saved = self.locals.clone();
                    for pat in &arm.pats {
                        collect_pat_binders(pat, &mut self.locals);
                    }
                    self.stmts(&arm.body);
                    self.locals = saved;
                }
            }
            MonoStmtKind::For {
                init,
                cond,
                post,
                body,
            } => {
                self.stmts_in_scope(init);
                self.visit_expr(cond);
                self.stmts_in_scope(post);
                self.stmts_in_scope(body);
            }
            _ => walk_stmt(self, stmt),
        }
    }

    fn visit_expr(&mut self, expr: &MonoExpr<'db>) {
        match &expr.kind {
            MonoExprKind::Var(id) => self.capture(id),
            MonoExprKind::Lambda { params, body, .. } => {
                let saved = self.locals.clone();
                self.locals
                    .extend(params.iter().map(|param| param.name.clone()));
                self.stmts(body);
                self.locals = saved;
            }
            MonoExprKind::Match { scrutinee, arms } => {
                self.visit_expr(scrutinee);
                for arm in arms {
                    let saved = self.locals.clone();
                    collect_pat_binders(&arm.pat, &mut self.locals);
                    self.visit_expr(&arm.expr);
                    self.locals = saved;
                }
            }
            _ => walk_expr(self, expr),
        }
    }
}
