use super::*;

struct VariantAbiMethod<'a, 'db> {
    class: &'a str,
    method: &'a str,
    main: Ty<'db>,
    extra: Vec<Ty<'db>>,
}

impl<'db> Driver<'db> {
    pub(super) fn specialize_derived_abi(
        &mut self,
        key: DerivedAbiKey<'db>,
        span: Span<'db>,
        depth: usize,
    ) -> Option<String> {
        if let Some(name) = self.derived_abis.get(&key) {
            return Some(name.clone());
        }
        if !self.ensure_specialization_type_size(&[key.target_ty], Some(span)) {
            return None;
        }

        let family = match key.family {
            DerivedAbiFamily::Attribs => "ABIAttribs",
            DerivedAbiFamily::Decode => "ABIDecode",
            DerivedAbiFamily::Encode => "ABIEncode",
        };
        let PredKind::InClass {
            class: ClassId::User(class),
            ..
        } = key.pred.kind(self.db)
        else {
            return None;
        };
        let mut base = format!(
            "Derived_{family}_{}_{}_{}",
            key.method,
            def_hash_suffix(self.db, key.adt),
            def_hash_suffix(self.db, *class),
        );
        if !key.sub_evidence.is_empty() {
            base.push('_');
            base.push_str(&evidence_hash_suffix(self.db, &key.sub_evidence));
        }
        let name = specialize_name(self.db, &base, &[key.target_ty]);
        self.derived_abis.insert(key.clone(), name.clone());
        self.derived_abi_order.push(key.clone());

        let variants = self.adts.get(&key.adt)?.adt.ctors(self.db).len() >= 2
            && self.abi_visible_def(&key, "ABIEncode").is_some();
        let fun = if variants {
            self.build_variant_abi_function(&key, &name, span, depth)
        } else {
            match key.family {
                DerivedAbiFamily::Attribs => {
                    self.build_derived_abi_attribs_function(&key, &name, span, depth)
                }
                DerivedAbiFamily::Decode => {
                    self.build_derived_abi_decode_function(&key, &name, span, depth)
                }
                DerivedAbiFamily::Encode => None,
            }
        };
        let Some(fun) = fun else {
            let mut context = format!("cannot generate {family}.{}", key.method);
            if variants && self.abi_visible_def(&key, "SigString").is_none() {
                context.push_str(
                    ": import std.dispatch in the ADT's defining module to make SigString visible",
                );
            }
            self.diagnostics.push(SpecializeDiagnostic {
                kind: SpecializeDiagnosticKind::UnsupportedEvidence { context },
                span: Some(span),
            });
            self.derived_abis.remove(&key);
            self.derived_abi_order.retain(|candidate| candidate != &key);
            return None;
        };
        self.derived_abi_funs.insert(key, fun);
        Some(name)
    }

    fn build_derived_abi_attribs_function(
        &mut self,
        key: &DerivedAbiKey<'db>,
        name: &str,
        span: Span<'db>,
        depth: usize,
    ) -> Option<MonoFunction<'db>> {
        let PredKind::InClass {
            class: ClassId::User(class),
            main,
            args,
        } = key.pred.kind(self.db)
        else {
            return None;
        };
        if !args.is_empty() || !matches!(key.method.as_str(), "headSize" | "isStatic") {
            return None;
        }
        if key.word_reader.is_some() {
            return None;
        }
        let TyKind::Named {
            ctor:
                TyCtor::User(UserTyCtor {
                    def: main_adt,
                    kind: UserTyCtorKind::Adt,
                }),
            args: main_args,
        } = main.kind(self.db)
        else {
            return None;
        };
        if *main_adt != key.adt {
            return None;
        }
        let adt_info = self.adts.get(&key.adt)?.clone();
        let generic = self.resolve_generic_instance(&adt_info, *main)?;
        let TyKind::Function { params, ret } =
            strip_comptime_ty(self.db, key.target_ty).kind(self.db)
        else {
            return None;
        };
        let [proxy_main] = params.as_slice() else {
            return None;
        };
        let TyKind::Named {
            ctor: proxy_ctor,
            args: proxy_args,
        } = strip_comptime_ty(self.db, *proxy_main).kind(self.db)
        else {
            return None;
        };
        if proxy_args.as_slice() != [*main] {
            return None;
        }
        let proxy_rep = Ty::named(self.db, *proxy_ctor, vec![generic.rep]);
        let delegated_pred =
            Pred::in_class(self.db, ClassId::User(*class), generic.rep, Vec::new());
        let delegated_ty = Ty::function(self.db, vec![proxy_rep], *ret);
        let givens = main_args
            .iter()
            .map(|arg| Pred::in_class(self.db, ClassId::User(*class), *arg, Vec::new()))
            .collect();
        let evidence =
            self.solve_derived_abi_pred_with_givens(key, delegated_pred, givens, Some(span))?;
        let delegated =
            self.resolve_class_method_call(&key.method, evidence, delegated_ty, span, depth + 1)?;

        let param = MonoParam {
            name: "_ty".to_owned(),
            mode: ParamMode::Runtime,
            ty: MonoTy::new_unchecked(*proxy_main),
            span,
        };
        let proxy = MonoExpr {
            span,
            ty: MonoTy::new_unchecked(proxy_rep),
            kind: MonoExprKind::Proxy(MonoTy::new_unchecked(generic.rep)),
        };
        let call = MonoExpr {
            span,
            ty: MonoTy::new_unchecked(*ret),
            kind: MonoExprKind::Call {
                callee: MonoId {
                    name: delegated,
                    ty: MonoTy::new_unchecked(delegated_ty),
                    span,
                },
                args: vec![proxy],
                origin: MonoCallOrigin::ByName,
            },
        };
        Some(MonoFunction {
            origin: MonoFunctionOrigin::DerivedGeneric {
                adt: key.adt,
                method: format!("ABIAttribs.{}", key.method),
            },
            source: None,
            shadowed_top_level: None,
            name: name.to_owned(),
            span,
            params: vec![param],
            ret: MonoTy::new_unchecked(*ret),
            comptime_obligations: Vec::new(),
            body: vec![MonoStmt {
                span,
                kind: MonoStmtKind::Return(Some(call)),
            }],
        })
    }

    fn build_derived_abi_decode_function(
        &mut self,
        key: &DerivedAbiKey<'db>,
        name: &str,
        span: Span<'db>,
        depth: usize,
    ) -> Option<MonoFunction<'db>> {
        let PredKind::InClass {
            class: ClassId::User(class),
            main: decoder,
            args,
        } = key.pred.kind(self.db)
        else {
            return None;
        };
        let [decoded] = args.as_slice() else {
            return None;
        };
        if key.method != "decode" {
            return None;
        }
        let TyKind::Named {
            ctor: decoder_ctor,
            args: decoder_args,
        } = decoder.kind(self.db)
        else {
            return None;
        };
        let [decoder_decoded, reader] = decoder_args.as_slice() else {
            return None;
        };
        if decoder_decoded != decoded {
            return None;
        }
        let TyCtor::User(decoder_user) = decoder_ctor else {
            return None;
        };
        let TyKind::Named {
            ctor:
                TyCtor::User(UserTyCtor {
                    def: decoded_adt,
                    kind: UserTyCtorKind::Adt,
                }),
            args: decoded_args,
        } = decoded.kind(self.db)
        else {
            return None;
        };
        if *decoded_adt != key.adt {
            return None;
        }
        let word_reader = key.word_reader?;
        let decoder_info = self.adts.get(&decoder_user.def)?.clone();
        let decoder_ctor_name = decoder_info.adt.ctors(self.db).first().map(|ctor| {
            format!(
                "{}_{}",
                decoder_user
                    .def
                    .name(self.db)
                    .unwrap_or_else(|| "ABIDecoder".to_owned()),
                ident_text(self.db, &ctor.name),
            )
        })?;

        let adt_info = self.adts.get(&key.adt)?.clone();
        let generic = self.resolve_generic_instance(&adt_info, *decoded)?;
        let rep_decoder = Ty::named(self.db, *decoder_ctor, vec![generic.rep, *reader]);
        let delegated_pred = Pred::in_class(
            self.db,
            ClassId::User(*class),
            rep_decoder,
            vec![generic.rep],
        );
        let TyKind::Function { params, ret } =
            strip_comptime_ty(self.db, key.target_ty).kind(self.db)
        else {
            return None;
        };
        let [decoder_param, head_offset] = params.as_slice() else {
            return None;
        };
        if *decoder_param != *decoder || *ret != *decoded {
            return None;
        }

        let delegated_ty = Ty::function(self.db, vec![rep_decoder, *head_offset], generic.rep);
        let mut givens = Vec::with_capacity(decoded_args.len() + 1);
        givens.push(Pred::in_class(
            self.db,
            ClassId::User(word_reader),
            *reader,
            Vec::new(),
        ));
        givens.extend(decoded_args.iter().map(|arg| {
            let param_decoder = Ty::named(self.db, *decoder_ctor, vec![*arg, *reader]);
            Pred::in_class(self.db, ClassId::User(*class), param_decoder, vec![*arg])
        }));
        let evidence =
            self.solve_derived_abi_pred_with_givens(key, delegated_pred, givens, Some(span))?;
        let delegated =
            self.resolve_class_method_call("decode", evidence, delegated_ty, span, depth + 1)?;
        let to_ty = Ty::function(self.db, vec![generic.rep], *decoded);
        let to = self.resolve_class_method_call("to", generic.evidence, to_ty, span, depth + 1)?;

        let ptr_param = MonoParam {
            name: "_ptr".to_owned(),
            mode: ParamMode::Runtime,
            ty: MonoTy::new_unchecked(*decoder_param),
            span,
        };
        let head_param = MonoParam {
            name: "_headOffset".to_owned(),
            mode: ParamMode::Runtime,
            ty: MonoTy::new_unchecked(*head_offset),
            span,
        };
        let rdr_id = MonoId {
            name: "_rdr".to_owned(),
            ty: MonoTy::new_unchecked(*reader),
            span,
        };
        let rep_ptr = MonoExpr {
            span,
            ty: MonoTy::new_unchecked(rep_decoder),
            kind: MonoExprKind::Con {
                ctor: MonoId {
                    name: decoder_ctor_name.clone(),
                    ty: MonoTy::new_unchecked(rep_decoder),
                    span,
                },
                args: vec![MonoExpr {
                    span,
                    ty: MonoTy::new_unchecked(*reader),
                    kind: MonoExprKind::Var(rdr_id.clone()),
                }],
            },
        };
        let head_arg = MonoExpr {
            span,
            ty: MonoTy::new_unchecked(*head_offset),
            kind: MonoExprKind::Var(MonoId {
                name: head_param.name.clone(),
                ty: head_param.ty,
                span,
            }),
        };
        let decoded_rep = MonoExpr {
            span,
            ty: MonoTy::new_unchecked(generic.rep),
            kind: MonoExprKind::Call {
                callee: MonoId {
                    name: delegated,
                    ty: MonoTy::new_unchecked(delegated_ty),
                    span,
                },
                args: vec![rep_ptr, head_arg],
                origin: MonoCallOrigin::ByName,
            },
        };
        let result = MonoExpr {
            span,
            ty: MonoTy::new_unchecked(*decoded),
            kind: MonoExprKind::Call {
                callee: MonoId {
                    name: to,
                    ty: MonoTy::new_unchecked(to_ty),
                    span,
                },
                args: vec![decoded_rep],
                origin: MonoCallOrigin::ByName,
            },
        };
        let ptr = MonoExpr {
            span,
            ty: ptr_param.ty,
            kind: MonoExprKind::Var(MonoId {
                name: ptr_param.name.clone(),
                ty: ptr_param.ty,
                span,
            }),
        };
        let pat = MonoPat {
            span,
            ty: MonoTy::new_unchecked(*decoder),
            kind: MonoPatKind::Con {
                ctor: MonoId {
                    name: decoder_ctor_name,
                    ty: MonoTy::new_unchecked(*decoder),
                    span,
                },
                args: vec![MonoPat {
                    span,
                    ty: MonoTy::new_unchecked(*reader),
                    kind: MonoPatKind::Var(rdr_id),
                }],
            },
        };
        Some(MonoFunction {
            origin: MonoFunctionOrigin::DerivedGeneric {
                adt: key.adt,
                method: "ABIDecode.decode".to_owned(),
            },
            source: None,
            shadowed_top_level: None,
            name: name.to_owned(),
            span,
            params: vec![ptr_param, head_param],
            ret: MonoTy::new_unchecked(*decoded),
            comptime_obligations: Vec::new(),
            body: vec![MonoStmt {
                span,
                kind: MonoStmtKind::Match {
                    scrutinees: vec![ptr],
                    arms: vec![MonoArm {
                        span,
                        pats: vec![pat],
                        body: vec![MonoStmt {
                            span,
                            kind: MonoStmtKind::Return(Some(result)),
                        }],
                    }],
                },
            }],
        })
    }

    fn solve_derived_abi_pred_with_givens(
        &mut self,
        key: &DerivedAbiKey<'db>,
        pred: Pred<'db>,
        givens: Vec<Pred<'db>>,
        span: Option<Span<'db>>,
    ) -> Option<Evidence<'db>> {
        if givens.len() != key.sub_evidence.len() {
            return None;
        }
        let adt_module = self.adts.get(&key.adt)?.module;
        let base = self.try_module_trait_env(adt_module).or_else(|| {
            self.push_missing_module_trait_env(span);
            None
        })?;
        let env = trait_env_with_givens(self.db, base, givens.clone());
        let Solution::Unique { evidence, .. } = solve(self.db, env, canonical_goal(self.db, pred))
        else {
            return None;
        };
        let bindings = givens
            .into_iter()
            .zip(key.sub_evidence.iter().cloned())
            .collect::<Vec<_>>();
        Some(self.elaborate_evidence(adt_module, evidence, &bindings))
    }
}

impl<'db> Driver<'db> {
    fn abi_visible_def(&self, key: &DerivedAbiKey<'db>, name: &str) -> Option<DefId<'db>> {
        let module = self.adts.get(&key.adt)?.module;
        let id = module_id_for_source_file(self.db, module.def_id_value(self.db).file(self.db))?;
        let surface = nameres::module_import_surface(self.db, id);
        let resolution = surface.types.get(name).or_else(|| {
            surface
                .item_scope
                .as_ref()?
                .types
                .get(name)
                .map(|entry| &entry.resolution)
        })?;
        match resolution {
            hir_nameres::Resolution::Def { def, .. } => Some(*def),
            _ => None,
        }
    }

    fn variant_abi_givens(&self, key: &DerivedAbiKey<'db>) -> Option<Vec<Pred<'db>>> {
        let PredKind::InClass { main, args, class } = key.pred.kind(self.db) else {
            return None;
        };
        let (main, decoder) = if key.family == DerivedAbiFamily::Decode {
            let TyKind::Named {
                ctor,
                args: dec_args,
            } = main.kind(self.db)
            else {
                return None;
            };
            (*args.first()?, Some((*ctor, *dec_args.get(1)?)))
        } else {
            (*main, None)
        };
        let TyKind::Named { args, .. } = main.kind(self.db) else {
            return None;
        };
        let mut givens = Vec::new();
        if let Some((_, reader)) = decoder {
            givens.push(Pred::in_class(
                self.db,
                ClassId::User(key.word_reader?),
                reader,
                vec![],
            ));
        }
        for arg in args {
            match key.family {
                DerivedAbiFamily::Attribs => {
                    givens.push(Pred::in_class(self.db, *class, *arg, vec![]))
                }
                DerivedAbiFamily::Encode => {
                    for name in ["ABIAttribs", "ABIEncode", "SigString"] {
                        givens.push(Pred::in_class(
                            self.db,
                            ClassId::User(self.abi_visible_def(key, name)?),
                            *arg,
                            vec![],
                        ));
                    }
                }
                DerivedAbiFamily::Decode => {
                    let (ctor, reader) = decoder?;
                    givens.push(Pred::in_class(
                        self.db,
                        *class,
                        Ty::named(self.db, ctor, vec![*arg, reader]),
                        vec![*arg],
                    ));
                    for name in ["SigString", "ABIAttribs"] {
                        givens.push(Pred::in_class(
                            self.db,
                            ClassId::User(self.abi_visible_def(key, name)?),
                            *arg,
                            vec![],
                        ));
                    }
                }
            }
        }
        Some(givens)
    }

    fn variant_abi_method(
        &mut self,
        key: &DerivedAbiKey<'db>,
        target: VariantAbiMethod<'_, 'db>,
        args: Vec<MonoExpr<'db>>,
        ret: Ty<'db>,
        span: Span<'db>,
        depth: usize,
    ) -> Option<MonoExpr<'db>> {
        let pred = Pred::in_class(
            self.db,
            ClassId::User(self.abi_visible_def(key, target.class)?),
            target.main,
            target.extra,
        );
        let evidence = self.solve_derived_abi_pred_with_givens(
            key,
            pred,
            self.variant_abi_givens(key)?,
            Some(span),
        )?;
        let ty = Ty::function(self.db, args.iter().map(|a| a.ty.ty()).collect(), ret);
        let name = self.resolve_class_method_call(target.method, evidence, ty, span, depth + 1)?;
        Some(abi_call(
            self.db,
            name,
            args,
            ret,
            span,
            MonoCallOrigin::ByName,
        ))
    }

    fn variant_abi_helper(
        &mut self,
        key: &DerivedAbiKey<'db>,
        helper: &str,
        args: Vec<MonoExpr<'db>>,
        ret: Ty<'db>,
        span: Span<'db>,
        depth: usize,
    ) -> Option<MonoExpr<'db>> {
        // Only select the canonical std helper; source modules may shadow its name.
        let def = self.functions.iter().find_map(|(def, info)| {
            (ident_text(self.db, &info.function.sig(self.db).name) == helper
                && (is_canonical_std_def_named(self.db, *def, helper)
                    || is_canonical_std_module_def_named(self.db, *def, "ABIGeneric", helper)))
            .then_some(*def)
        })?;
        let info = self.functions.get(&def)?.clone();
        let ty = Ty::function(self.db, args.iter().map(|a| a.ty.ty()).collect(), ret);
        let lowered = self.try_lower_normalized_function(&info)?;
        let mut subst = TySubst::default();
        if !subst.match_ty(self.db, lowered.scheme.body(self.db).ty(self.db), ty) {
            return None;
        }
        let givens = self.function_givens(&info, &lowered);
        let mut evidence_bindings = Vec::new();
        for pred in givens {
            let pred = subst.apply_pred(self.db, pred);
            let evidence = self.solve_derived_abi_pred_with_givens(
                key,
                pred,
                self.variant_abi_givens(key)?,
                Some(span),
            )?;
            evidence_bindings.push((pred, evidence));
        }
        let base = self.source_base_name(&info);
        let name = self.enqueue(
            SpecKey {
                def,
                ty,
                base_name: specialize_name(self.db, &base, &subst.specialization_args()),
                origin: MonoFunctionOrigin::Source,
                evidence_bindings,
            },
            depth + 1,
        );
        Some(abi_call(
            self.db,
            name,
            args,
            ret,
            span,
            MonoCallOrigin::Source(def),
        ))
    }

    fn variant_abi_proxy(
        &self,
        key: &DerivedAbiKey<'db>,
        ty: Ty<'db>,
        span: Span<'db>,
    ) -> Option<MonoExpr<'db>> {
        let proxy = self.abi_visible_def(key, "Proxy")?;
        let proxy_ty = Ty::named(
            self.db,
            TyCtor::User(UserTyCtor {
                def: proxy,
                kind: UserTyCtorKind::Adt,
            }),
            vec![ty],
        );
        Some(MonoExpr {
            span,
            ty: MonoTy::new_unchecked(proxy_ty),
            kind: MonoExprKind::Proxy(MonoTy::new_unchecked(ty)),
        })
    }

    fn variant_abi_attr(
        &mut self,
        key: &DerivedAbiKey<'db>,
        ty: Ty<'db>,
        method: &str,
        span: Span<'db>,
        depth: usize,
    ) -> Option<MonoExpr<'db>> {
        let proxy = self.variant_abi_proxy(key, ty, span)?;
        let ret = if method == "isStatic" {
            Ty::bool(self.db)
        } else {
            Ty::word(self.db)
        };
        self.variant_abi_method(
            key,
            VariantAbiMethod {
                class: "ABIAttribs",
                method,
                main: ty,
                extra: vec![],
            },
            vec![proxy],
            ret,
            span,
            depth,
        )
    }

    fn variant_abi_tag(
        &mut self,
        key: &DerivedAbiKey<'db>,
        ctor: &str,
        rep: Ty<'db>,
        span: Span<'db>,
        depth: usize,
    ) -> Option<MonoExpr<'db>> {
        let string_ty = Ty::named(
            self.db,
            TyCtor::User(UserTyCtor {
                def: self.abi_visible_def(key, "string")?,
                kind: UserTyCtorKind::Adt,
            }),
            vec![],
        );
        let proxy = self.variant_abi_proxy(key, rep, span)?;
        let sig = self.variant_abi_method(
            key,
            VariantAbiMethod {
                class: "SigString",
                method: "sigStr",
                main: rep,
                extra: vec![],
            },
            vec![proxy],
            string_ty,
            span,
            depth,
        )?;
        let text = |s: String| MonoExpr {
            span,
            ty: MonoTy::new_unchecked(string_ty),
            kind: MonoExprKind::Lit(LitKind::String(format!("{s:?}"))),
        };
        let sig = abi_binary(text(format!("{ctor}(")), BinOp::Add, sig, string_ty, span);
        let sig = abi_binary(sig, BinOp::Add, text(")".to_owned()), string_ty, span);
        Some(abi_call(
            self.db,
            "keccakLit".to_owned(),
            vec![sig],
            Ty::word(self.db),
            span,
            MonoCallOrigin::Builtin(MonoIntrinsic::KeccakLit),
        ))
    }

    fn build_variant_abi_function(
        &mut self,
        key: &DerivedAbiKey<'db>,
        name: &str,
        span: Span<'db>,
        depth: usize,
    ) -> Option<MonoFunction<'db>> {
        let PredKind::InClass { main, args, .. } = key.pred.kind(self.db) else {
            return None;
        };
        let main = if key.family == DerivedAbiFamily::Decode {
            *args.first()?
        } else {
            *main
        };
        let TyKind::Named { args: ty_args, .. } = main.kind(self.db) else {
            return None;
        };
        let info = self.adts.get(&key.adt)?.clone();
        let plan = derived_generic_plan(self.db, info.module, info.adt)?;
        let mut subst = TySubst::default();
        for (index, arg) in ty_args.iter().enumerate() {
            subst.insert_if_consistent(index as u32, *arg);
        }
        let reps = plan
            .from_arms
            .iter()
            .map(|arm| subst.apply_ty(self.db, arm.product_rep))
            .collect::<Vec<_>>();
        let TyKind::Function {
            params: param_tys,
            ret,
        } = strip_comptime_ty(self.db, key.target_ty).kind(self.db)
        else {
            return None;
        };
        let params = param_tys
            .iter()
            .enumerate()
            .map(|(i, ty)| MonoParam {
                name: format!("_arg{i}"),
                mode: ParamMode::Runtime,
                ty: MonoTy::new_unchecked(*ty),
                span,
            })
            .collect::<Vec<_>>();
        let param_exprs = params
            .iter()
            .map(|p| abi_var(&p.name, p.ty.ty(), span))
            .collect::<Vec<_>>();
        // A named ADT has one discriminant, regardless of the nesting of Generic's sum.
        let mut statics = Vec::new();
        let mut heads = Vec::new();
        for rep in &reps {
            statics.push(self.variant_abi_attr(key, *rep, "isStatic", span, depth)?);
            heads.push(self.variant_abi_attr(key, *rep, "headSize", span, depth)?);
        }
        let mut is_static = statics.remove(0);
        for value in statics {
            is_static = self.variant_abi_helper(
                key,
                "and",
                vec![is_static, value],
                Ty::bool(self.db),
                span,
                depth,
            )?;
        }
        let mut widest = heads.remove(0);
        for value in heads {
            widest = self.variant_abi_helper(
                key,
                "maxWord",
                vec![widest, value],
                Ty::word(self.db),
                span,
                depth,
            )?;
        }
        let inner_head = abi_binary(
            abi_word(self.db, 32, span),
            BinOp::Add,
            widest,
            Ty::word(self.db),
            span,
        );
        let body = match key.family {
            DerivedAbiFamily::Attribs => match key.method.as_str() {
                "isStatic" => vec![abi_return(is_static, span)],
                "headSize" => vec![MonoStmt {
                    span,
                    kind: MonoStmtKind::If {
                        cond: is_static,
                        then_body: vec![abi_return(inner_head, span)],
                        else_body: Some(vec![abi_return(abi_word(self.db, 32, span), span)]),
                    },
                }],
                _ => return None,
            },
            DerivedAbiFamily::Encode => {
                let [value, base, offset, tail] = param_exprs.as_slice() else {
                    return None;
                };
                let mut arms = Vec::new();
                for (arm, rep) in plan.from_arms.iter().zip(&reps) {
                    let vars = product_vars(self.db, *rep, arm.field_count as usize, span, "f")?;
                    let payload = product_expr_from_vars(self.db, &vars, *rep, span);
                    let tag = self.variant_abi_tag(key, &arm.ctor_name, *rep, span, depth)?;
                    let result = self.variant_abi_helper(
                        key,
                        "encodeVariant",
                        vec![
                            tag,
                            payload,
                            is_static.clone(),
                            inner_head.clone(),
                            base.clone(),
                            offset.clone(),
                            tail.clone(),
                        ],
                        Ty::word(self.db),
                        span,
                        depth,
                    )?;
                    let ctor = MonoId {
                        name: format!("{}_{}", key.adt.name(self.db)?, arm.ctor_name),
                        ty: MonoTy::new_unchecked(main),
                        span,
                    };
                    let pat = MonoPat {
                        span,
                        ty: MonoTy::new_unchecked(main),
                        kind: MonoPatKind::Con {
                            ctor,
                            args: vars.iter().map(|v| var_pattern(v, span)).collect(),
                        },
                    };
                    arms.push(MonoArm {
                        span,
                        pats: vec![pat],
                        body: vec![abi_return(result, span)],
                    });
                }
                vec![MonoStmt {
                    span,
                    kind: MonoStmtKind::Match {
                        scrutinees: vec![value.clone()],
                        arms,
                    },
                }]
            }
            DerivedAbiFamily::Decode => {
                let [ptr, offset] = param_exprs.as_slice() else {
                    return None;
                };
                let TyKind::Named {
                    ctor: decoder_ctor,
                    args: decoder_args,
                } = ptr.ty.ty().kind(self.db)
                else {
                    return None;
                };
                let reader = *decoder_args.get(1)?;
                let rdr = abi_var("_rdr", reader, span);
                let sum_reader = self.variant_abi_helper(
                    key,
                    "abiSumReader",
                    vec![rdr.clone(), offset.clone(), is_static],
                    reader,
                    span,
                    depth,
                )?;
                let sum_rdr = abi_var("_sumRdr", reader, span);
                let tag = self.variant_abi_method(
                    key,
                    VariantAbiMethod {
                        class: "WordReader",
                        method: "read",
                        main: reader,
                        extra: vec![],
                    },
                    vec![sum_rdr.clone()],
                    Ty::word(self.db),
                    span,
                    depth,
                )?;
                let generic = self.resolve_generic_instance(&info, main)?;
                let to_ty = Ty::function(self.db, vec![generic.rep], main);
                let to_name =
                    self.resolve_class_method_call("to", generic.evidence, to_ty, span, depth + 1)?;
                // Reject unrecognized tags instead of silently selecting the final variant.
                let invalid = self.derived_class_absurd_expr(main, span, depth)?;
                let mut dispatch = vec![abi_return(invalid, span)];
                for (arm, rep) in plan.from_arms.iter().zip(&reps).rev() {
                    let decoder_ty = Ty::named(self.db, *decoder_ctor, vec![*rep, reader]);
                    let decoder = MonoExpr {
                        span,
                        ty: MonoTy::new_unchecked(decoder_ty),
                        kind: MonoExprKind::Con {
                            ctor: MonoId {
                                name: "ABIDecoder_ABIDecoder".to_owned(),
                                ty: MonoTy::new_unchecked(decoder_ty),
                                span,
                            },
                            args: vec![sum_rdr.clone()],
                        },
                    };
                    let value = self.variant_abi_method(
                        key,
                        VariantAbiMethod {
                            class: "ABIDecode",
                            method: "decode",
                            main: decoder_ty,
                            extra: vec![*rep],
                        },
                        vec![decoder, abi_word(self.db, 32, span)],
                        *rep,
                        span,
                        depth,
                    )?;
                    let rep_value = wrap_sum_expr(
                        self.db,
                        value,
                        generic.rep,
                        arm.inr_depth,
                        arm.wraps_inl,
                        span,
                    );
                    let value = abi_call(
                        self.db,
                        to_name.clone(),
                        vec![rep_value],
                        main,
                        span,
                        MonoCallOrigin::ByName,
                    );
                    let expected = self.variant_abi_tag(key, &arm.ctor_name, *rep, span, depth)?;
                    let condition = abi_binary(
                        abi_var("_tag", Ty::word(self.db), span),
                        BinOp::Eq,
                        expected,
                        Ty::bool(self.db),
                        span,
                    );
                    dispatch = vec![MonoStmt {
                        span,
                        kind: MonoStmtKind::If {
                            cond: condition,
                            then_body: vec![abi_return(value, span)],
                            else_body: Some(dispatch),
                        },
                    }];
                }
                let mut decode_body = vec![
                    abi_let("_sumRdr", sum_reader, span),
                    abi_let("_tag", tag, span),
                ];
                decode_body.extend(dispatch);
                let pat = MonoPat {
                    span,
                    ty: ptr.ty,
                    kind: MonoPatKind::Con {
                        ctor: MonoId {
                            name: "ABIDecoder_ABIDecoder".to_owned(),
                            ty: ptr.ty,
                            span,
                        },
                        args: vec![MonoPat {
                            span,
                            ty: rdr.ty,
                            kind: MonoPatKind::Var(MonoId {
                                name: "_rdr".to_owned(),
                                ty: rdr.ty,
                                span,
                            }),
                        }],
                    },
                };
                vec![MonoStmt {
                    span,
                    kind: MonoStmtKind::Match {
                        scrutinees: vec![ptr.clone()],
                        arms: vec![MonoArm {
                            span,
                            pats: vec![pat],
                            body: decode_body,
                        }],
                    },
                }]
            }
        };
        Some(MonoFunction {
            origin: MonoFunctionOrigin::DerivedGeneric {
                adt: key.adt,
                method: format!(
                    "{}.{}",
                    match key.family {
                        DerivedAbiFamily::Attribs => "ABIAttribs",
                        DerivedAbiFamily::Encode => "ABIEncode",
                        DerivedAbiFamily::Decode => "ABIDecode",
                    },
                    key.method
                ),
            },
            source: None,
            shadowed_top_level: None,
            name: name.to_owned(),
            span,
            params,
            ret: MonoTy::new_unchecked(*ret),
            comptime_obligations: vec![],
            body,
        })
    }
}

fn abi_call<'db>(
    db: &'db dyn Db,
    name: String,
    args: Vec<MonoExpr<'db>>,
    ret: Ty<'db>,
    span: Span<'db>,
    origin: MonoCallOrigin<'db>,
) -> MonoExpr<'db> {
    let ty = Ty::function(db, args.iter().map(|a| a.ty.ty()).collect(), ret);
    MonoExpr {
        span,
        ty: MonoTy::new_unchecked(ret),
        kind: MonoExprKind::Call {
            callee: MonoId {
                name,
                ty: MonoTy::new_unchecked(ty),
                span,
            },
            args,
            origin,
        },
    }
}
fn abi_binary<'db>(
    lhs: MonoExpr<'db>,
    op: BinOp,
    rhs: MonoExpr<'db>,
    ty: Ty<'db>,
    span: Span<'db>,
) -> MonoExpr<'db> {
    MonoExpr {
        span,
        ty: MonoTy::new_unchecked(ty),
        kind: MonoExprKind::BinOp {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        },
    }
}
fn abi_var<'db>(name: &str, ty: Ty<'db>, span: Span<'db>) -> MonoExpr<'db> {
    let ty = MonoTy::new_unchecked(ty);
    MonoExpr {
        span,
        ty,
        kind: MonoExprKind::Var(MonoId {
            name: name.to_owned(),
            ty,
            span,
        }),
    }
}
fn abi_word<'db>(db: &'db dyn Db, value: u64, span: Span<'db>) -> MonoExpr<'db> {
    MonoExpr {
        span,
        ty: MonoTy::new_unchecked(Ty::word(db)),
        kind: MonoExprKind::Lit(LitKind::Number(value.to_string())),
    }
}
fn abi_return<'db>(value: MonoExpr<'db>, span: Span<'db>) -> MonoStmt<'db> {
    MonoStmt {
        span,
        kind: MonoStmtKind::Return(Some(value)),
    }
}
fn abi_let<'db>(name: &str, value: MonoExpr<'db>, span: Span<'db>) -> MonoStmt<'db> {
    MonoStmt {
        span,
        kind: MonoStmtKind::Let {
            mode: LetMode::Runtime,
            id: MonoId {
                name: name.to_owned(),
                ty: value.ty,
                span,
            },
            ty: Some(value.ty),
            init: Some(value),
        },
    }
}
