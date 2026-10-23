use hir::{
    anchor::DefId,
    ast::item::{ContractDef, ContractItem, FuncKind, Item, Module},
    diag::{Diagnostic, DiagnosticCode, LabelSpan},
    nameres as hir_nameres,
    span::Spanned,
};
use nameres::{LibraryId, module_id_for_source_file};
use parser::parse_file_to_hir;
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
    abi::{
        AbiAdtEvidence, AbiParam, AbiSelector, AbiSignature, AbiType, abi_outputs, abi_params,
        abi_selector, abi_type_contains_user_adt, contract_diag_unsupported_abi_type,
        method_signature_string,
    },
    dispatch_abi::{MethodParts, constructor_abi_roots, method_abi_roots},
    helpers::{
        find_contract_by_def, function_type_vars, ident_text, lower_normalized_function,
        param_names, resolve_contract_item_types, type_var_bindings,
    },
    std_abi::{AbiClass, StdAbiType},
};
use crate::{
    ClassId, ClauseOrigin, Db, DerivedClauseKind, Pred, PredKind, TraitEnvId, Ty, TyCtor, TyKind,
    TypeckDiagnostic, UserTyCtorKind,
    solver::{
        AbiEvidenceRole, Evidence, Solution, canonical_goal, solve_report,
        solver_answer_is_closed_over_goal,
    },
};

const MAX_DERIVED_ABI_EVIDENCE: usize = 1_024;

/// Typed dispatch/ABI surface for one contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub struct DispatchSurface<'db> {
    /// Owning contract definition.
    pub contract: DefId<'db>,
    /// Contract name.
    pub name: String,
    /// Public methods eligible for selector dispatch.
    pub methods: Vec<DispatchMethod<'db>>,
    /// Constructor entry. A missing source constructor is represented as an
    /// implicit non-payable unit constructor.
    pub constructor: DispatchConstructor,
    /// Fallback entry. A missing source fallback is represented as the default
    /// non-payable unit fallback.
    pub fallback: DispatchFallback<'db>,
    /// Canonical-ABI diagnostics produced specifically by the constructor.
    /// These remain compilation errors even when a source runtime `main`
    /// suppresses generated method dispatch.
    pub constructor_abi_diagnostics: Vec<Diagnostic>,
    /// Diagnostics produced while building the surface.
    pub diagnostics: Vec<Diagnostic>,
}

/// One public method in the dispatch surface.
#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub struct DispatchMethod<'db> {
    /// Function definition.
    pub def: DefId<'db>,
    /// Source declaration index within the contract.
    pub source_index: usize,
    /// Source method name.
    pub name: String,
    /// Whether the method is payable.
    pub payable: bool,
    /// ABI selector preimage, e.g. `transfer(address,uint256)`.
    pub signature: String,
    /// First four bytes of `keccak256(signature)`.
    pub selector: AbiSelector,
    /// ABI input parameters.
    pub inputs: Vec<AbiParam>,
    /// ABI output parameters.
    pub outputs: Vec<AbiParam>,
}

/// Constructor dispatch/ABI entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub enum DispatchConstructor {
    /// No source constructor: implicit non-payable unit constructor.
    Implicit,
    /// Source constructor declaration.
    Explicit {
        /// Source declaration index within the contract.
        source_index: usize,
        /// Whether deployment may receive value.
        payable: bool,
        /// ABI input parameters.
        inputs: Vec<AbiParam>,
    },
}

/// Fallback dispatch/ABI entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub enum DispatchFallback<'db> {
    /// No source fallback: default non-payable unit fallback.
    Default,
    /// Source fallback declaration.
    Explicit {
        /// Source fallback definition.
        def: DefId<'db>,
        /// Source declaration index within the contract.
        source_index: usize,
        /// Whether fallback calls may receive value.
        payable: bool,
        /// ABI input parameters. Valid Solcore fallbacks are unit.
        inputs: Vec<AbiParam>,
        /// ABI output parameters. Valid Solcore fallbacks are unit.
        outputs: Vec<AbiParam>,
    },
}

/// Returns the typed dispatch surface for one contract in `module`.
pub fn contract_dispatch_surface<'db>(
    db: &'db dyn Db,
    module: Module<'db>,
    contract: ContractDef<'db>,
) -> DispatchSurface<'db> {
    let _ = module;
    contract_dispatch_surface_by_def(db, contract.def_id_value(db))
}

/// Returns the typed dispatch surface for `contract` using the supplied HIR
/// module directly. This is useful for backend-generated HIR overlays whose
/// source file URL may intentionally mirror a user file.
pub fn contract_dispatch_surface_for_module<'db>(
    db: &'db dyn Db,
    module: Module<'db>,
    contract: ContractDef<'db>,
) -> DispatchSurface<'db> {
    let item_resolutions = resolve_contract_item_types(db, module);
    contract_dispatch_surface_with_resolutions(db, module, &item_resolutions, contract)
}

#[salsa::tracked]
fn contract_dispatch_surface_by_def<'db>(
    db: &'db dyn Db,
    contract_def: DefId<'db>,
) -> DispatchSurface<'db> {
    let module = parse_file_to_hir(db, contract_def.file(db)).module(db);
    let Some(contract) = find_contract_by_def(db, module, contract_def) else {
        return DispatchSurface {
            contract: contract_def,
            name: contract_def
                .name(db)
                .unwrap_or_else(|| "Contract".to_owned()),
            methods: Vec::new(),
            constructor: DispatchConstructor::Implicit,
            fallback: DispatchFallback::Default,
            constructor_abi_diagnostics: Vec::new(),
            diagnostics: Vec::new(),
        };
    };
    let item_resolutions = resolve_contract_item_types(db, module);
    contract_dispatch_surface_with_resolutions(db, module, &item_resolutions, contract)
}

/// Returns diagnostics for every contract dispatch surface in a module.
pub fn module_contract_diagnostics<'db>(db: &'db dyn Db, module: Module<'db>) -> Vec<Diagnostic> {
    module
        .items(db)
        .iter()
        .filter_map(|item| match item {
            Item::ContractDef(contract) => Some(*contract),
            _ => None,
        })
        .flat_map(|contract| {
            let dispatch_generated = contract_needs_generated_dispatch(db, contract);
            let surface = contract_dispatch_surface(db, module, contract);
            let mut diagnostics = if dispatch_generated {
                surface.diagnostics
            } else {
                let mut diagnostics = surface
                    .diagnostics
                    .into_iter()
                    .filter(|diagnostic| diagnostic.code.as_deref() != Some("SC0231"))
                    .collect::<Vec<_>>();
                diagnostics.extend(surface.constructor_abi_diagnostics);
                diagnostics
            };
            diagnostics.extend(contract_runtime_main_diagnostics(db, contract));
            diagnostics
        })
        .filter(|diagnostic| {
            matches!(
                diagnostic.code.as_deref(),
                Some("SC0230" | "SC0231" | "SC0232" | "SC0233" | "SC0235" | "SC0236")
            )
        })
        .collect()
}

pub(crate) fn module_manual_generic_abi_diagnostics<'db>(
    db: &'db dyn Db,
    module: Module<'db>,
    prepared_module: Module<'db>,
    trait_env: TraitEnvId<'db>,
) -> Vec<Diagnostic> {
    let clauses = trait_env.clauses(db);
    let manual_evidence = clauses
        .iter()
        .filter_map(|clause| {
            let ClauseOrigin::Instance { def: instance, .. } = &clause.origin else {
                return None;
            };
            if instance
                .fingerprint(db)
                .as_deref()
                .is_some_and(|fingerprint| {
                    fingerprint.starts_with("solcore.generated.std_dispatch.")
                })
            {
                return None;
            }
            if module_id_for_source_file(db, instance.file(db))
                .is_some_and(|module| matches!(module.library(db), LibraryId::Std))
            {
                return None;
            }
            let PredKind::InClass {
                class: ClassId::User(class),
                main,
                ..
            } = clause.head.kind(db)
            else {
                return None;
            };
            let class = AbiClass::from_def(db, *class)?;
            if class.owns_instance(db, *instance) {
                return None;
            }
            let generic_adt = if class == AbiClass::Generic {
                match main.kind(db) {
                    TyKind::Named {
                        ctor: TyCtor::User(user),
                        ..
                    } if user.kind == UserTyCtorKind::Adt => Some(user.def),
                    _ => return None,
                }
            } else {
                None
            };
            Some((*instance, class, generic_adt))
        })
        .collect::<Vec<_>>();
    if manual_evidence.is_empty() {
        return Vec::new();
    }
    let manual_instances = manual_evidence
        .iter()
        .map(|(instance, class, _)| (*instance, *class))
        .collect::<FxHashMap<_, _>>();
    let classes = clauses
        .iter()
        .filter_map(|clause| {
            let PredKind::InClass {
                class: ClassId::User(class),
                ..
            } = clause.head.kind(db)
            else {
                return None;
            };
            Some((AbiClass::from_def(db, *class)?, *class))
        })
        .collect::<FxHashMap<_, _>>();

    let item_resolutions = resolve_contract_item_types(db, module);
    let mut diagnostics = Vec::new();
    for item in module.items(db) {
        let Item::ContractDef(contract) = *item else {
            continue;
        };
        let dispatch_generated = contract_needs_generated_dispatch(db, contract);
        let contract_name = ident_text(db, &contract.name_elem(db));
        let contract_type_vars =
            type_var_bindings(contract.def_id_value(db), contract.ty_param_elems(db));
        for item in contract.items(db) {
            let ContractItem::FunctionDef(function) = *item else {
                continue;
            };
            let sig = function.sig(db);
            let abi_context = match function.kind(db) {
                FuncKind::Constructor => Some("constructor".to_owned()),
                FuncKind::Function if sig.public.is_some() && dispatch_generated => {
                    Some(format!("public function `{}`", ident_text(db, &sig.name)))
                }
                FuncKind::Function | FuncKind::Fallback => None,
            };
            let Some(abi_context) = abi_context else {
                continue;
            };
            let type_vars =
                function_type_vars(db, &contract_type_vars, function.def_id_value(db), sig);
            let lowered = lower_normalized_function(
                db,
                module,
                &item_resolutions,
                contract.def_id_value(db),
                function,
                &type_vars,
            );
            let mut exposed_tys = lowered.params.clone();
            if function.kind(db) == FuncKind::Function {
                exposed_tys.push(lowered.ret);
            }
            let args = crate::lower::product_ty(db, lowered.params.iter().copied());
            let roots = if function.kind(db) == FuncKind::Function {
                let name = crate::contract_dispatch_name_type_name(
                    &contract_name,
                    &ident_text(db, &sig.name),
                );
                let name_ty = prepared_module
                    .items(db)
                    .iter()
                    .find_map(|item| match item {
                        Item::AdtDef(adt)
                            if ident_text(db, &adt.name_elem(db)) == name
                                && adt.def_id_value(db).fingerprint(db).as_deref()
                                    == Some("solcore.generated.std_dispatch.name_type") =>
                        {
                            Some(Ty::named(
                                db,
                                TyCtor::User(crate::UserTyCtor {
                                    def: adt.def_id_value(db),
                                    kind: UserTyCtorKind::Adt,
                                }),
                                Vec::new(),
                            ))
                        }
                        _ => None,
                    });
                let payability = if sig.payable.is_some() {
                    StdAbiType::Payable
                } else {
                    StdAbiType::NonPayable
                };
                method_abi_roots(
                    db,
                    &classes,
                    MethodParts {
                        name: name_ty.unwrap_or_else(|| Ty::unknown(db)),
                        payability: payability
                            .ty(db, Vec::new())
                            .unwrap_or_else(|| Ty::unknown(db)),
                        args,
                        returns: lowered.ret,
                        implementation: Ty::function(db, lowered.params.clone(), lowered.ret),
                    },
                )
            } else if !lowered.params.is_empty() {
                // Empty constructors do not emit an abi_decode call.
                constructor_abi_roots(db, &classes, args)
            } else {
                Vec::new()
            };
            let mut manual = ManualAbiEvidence::new(db, &manual_instances);
            for (role, goal) in roots {
                let report = solve_report(db, trait_env, canonical_goal(db, goal));
                if report.exhausted {
                    continue;
                }
                if let Solution::Unique { subst, evidence } = report.solution
                    && solver_answer_is_closed_over_goal(db, goal, trait_env, &subst, &evidence)
                {
                    manual.collect(&evidence, role);
                }
            }
            if let Some(pred) = manual.exhausted_derived {
                diagnostics.push(
                    TypeckDiagnostic::SolverFuelExhausted {
                        span: LabelSpan::from_span(db, sig.span),
                        pred: crate::display::display_pred_source(db, pred, &[]),
                    }
                    .lower(),
                );
            }
            for (instance, class, generic_adt) in &manual_evidence {
                if *class != AbiClass::Generic && !manual.selected.contains(instance) {
                    continue;
                }
                if generic_adt.is_some_and(|adt| {
                    !exposed_tys
                        .iter()
                        .any(|ty| abi_type_contains_user_adt(db, *ty, adt))
                }) {
                    continue;
                }
                let class_name = class.name();
                let subject = generic_adt.map_or_else(
                    || format!("visible manual `{class_name}` evidence"),
                    |adt| {
                        let adt_name = adt.name(db).unwrap_or_else(|| "<anonymous ADT>".to_owned());
                        format!("`{adt_name}` with visible manual `Generic` evidence")
                    },
                );
                diagnostics.push(
                    Diagnostic::error(format!(
                        "{abi_context} ABI for contract `{contract_name}` cannot use {subject}"
                    ))
                    .with_code("SC0231")
                    .with_primary_label(
                        db,
                        sig.span,
                        Some("external ABI evidence must be compiler-owned and canonical"),
                    )
                    .with_note(format!(
                        "impl `{}` can override canonical `{class_name}` behavior",
                        instance
                            .name(db)
                            .unwrap_or_else(|| class_name.to_string())
                    ))
                    .with_help(
                        "remove the visible manual ABI impl or keep this declaration out of the external ABI",
                    ),
                );
            }
        }
    }
    diagnostics
}

/// Follows selected dictionaries, including representation and superclass
/// subproofs. Signature roots own SigString and ABITypeString evidence for method
/// selectors and variant tags; execution roots own encoding, decoding, and
/// layout evidence, including ABIElementAttribs for array value layouts.
/// All roots of one declaration share the same visited set and proof budget.
struct ManualAbiEvidence<'db, 'a> {
    db: &'db dyn Db,
    manual_instances: &'a FxHashMap<DefId<'db>, AbiClass>,
    selected: FxHashSet<DefId<'db>>,
    visited_derived: FxHashSet<(DerivedClauseKind<'db>, Pred<'db>)>,
    exhausted_derived: Option<Pred<'db>>,
}

impl<'db, 'a> ManualAbiEvidence<'db, 'a> {
    fn new(db: &'db dyn Db, manual_instances: &'a FxHashMap<DefId<'db>, AbiClass>) -> Self {
        Self {
            db,
            manual_instances,
            selected: FxHashSet::default(),
            visited_derived: FxHashSet::default(),
            exhausted_derived: None,
        }
    }

    fn collect(&mut self, evidence: &Evidence<'db>, role: AbiEvidenceRole) {
        match evidence {
            Evidence::Instance {
                instance,
                sub_evidence,
                ..
            } => {
                if let Some(class) = self.manual_instances.get(instance)
                    && class.evidence_role() == Some(role)
                {
                    self.selected.insert(*instance);
                }
                for evidence in sub_evidence {
                    self.collect(evidence, role);
                }
            }
            Evidence::Derived {
                kind,
                pred,
                sub_evidence,
            } => {
                for evidence in sub_evidence {
                    self.collect(evidence, role);
                }
                // Derived ABI bodies resolve their representation dictionaries in
                // the defining module. Bound that additional proof graph just like
                // the solver bounds type-growing tables; cyclic source layouts
                // retain their existing unsupported-ABI diagnostic.
                if role == AbiEvidenceRole::Codec
                    && matches!(
                        kind,
                        DerivedClauseKind::AbiAttribs { .. }
                            | DerivedClauseKind::AbiEncode { .. }
                            | DerivedClauseKind::AbiDecode { .. }
                    )
                    && !self.visited_derived.contains(&(*kind, *pred))
                {
                    if self.visited_derived.len() == MAX_DERIVED_ABI_EVIDENCE {
                        self.exhausted_derived.get_or_insert(*pred);
                    } else {
                        self.visited_derived.insert((*kind, *pred));
                        match crate::solver::derived_abi_delegated_evidence(self.db, evidence) {
                            Ok(Some(dependencies)) => {
                                for (role, dependency) in dependencies {
                                    self.collect(&dependency, role);
                                }
                            }
                            Err(pred) => {
                                self.exhausted_derived.get_or_insert(pred);
                            }
                            Ok(None) => {}
                        }
                    }
                }
            }
            Evidence::Superclass { child, .. } => self.collect(child, role),
            Evidence::Builtin { .. } => {}
        }
    }
}

fn visible_abi_adt_evidence<'db>(db: &'db dyn Db, module: Module<'db>) -> AbiAdtEvidence<'db> {
    let trait_env =
        if let Some(module_id) = module_id_for_source_file(db, module.def_id_value(db).file(db)) {
            crate::trait_env_for_module(db, module_id)
        } else {
            let resolution = hir_nameres::resolve_module(db, module);
            crate::trait_env_from_module_resolution(db, module, &resolution)
        };
    let mut manual_generic_adts = FxHashSet::default();
    let mut derived_abi_attribs = FxHashSet::default();
    let mut derived_abi_decodes = FxHashSet::default();

    for clause in trait_env.clauses(db) {
        match &clause.origin {
            ClauseOrigin::Derived(DerivedClauseKind::AbiAttribs { adt }) => {
                derived_abi_attribs.insert(*adt);
            }
            ClauseOrigin::Derived(DerivedClauseKind::AbiDecode { adt, .. }) => {
                derived_abi_decodes.insert(*adt);
            }
            ClauseOrigin::Instance { .. } => {
                let PredKind::InClass {
                    class: ClassId::User(class),
                    main,
                    ..
                } = clause.head.kind(db)
                else {
                    continue;
                };
                if AbiClass::from_def(db, *class) != Some(AbiClass::Generic) {
                    continue;
                }
                if let TyKind::Named {
                    ctor: TyCtor::User(user),
                    ..
                } = main.kind(db)
                    && user.kind == UserTyCtorKind::Adt
                {
                    manual_generic_adts.insert(user.def);
                }
            }
            ClauseOrigin::Builtin
            | ClauseOrigin::Derived(_)
            | ClauseOrigin::Given
            | ClauseOrigin::Superclass(_) => {}
        }
    }

    let derived_abi_adts = derived_abi_attribs
        .into_iter()
        .filter(|adt| derived_abi_decodes.contains(adt))
        .collect();
    AbiAdtEvidence::new(manual_generic_adts, derived_abi_adts)
}

fn contract_runtime_main_diagnostics<'db>(
    db: &'db dyn Db,
    contract: ContractDef<'db>,
) -> Vec<Diagnostic> {
    contract
        .items(db)
        .iter()
        .filter_map(|item| {
            let ContractItem::FunctionDef(function) = *item else {
                return None;
            };
            let sig = function.sig(db);
            (function.kind(db) == FuncKind::Function
                && ident_text(db, &sig.name) == "main"
                && !sig.params.atom().is_empty())
            .then(|| {
                Diagnostic::error("contract runtime `main` must not take parameters")
                    .with_code(DiagnosticCode::TYPECK_CONTRACT_RUNTIME_MAIN_ARITY)
                    .with_primary_label(
                        db,
                        sig.params.span(db),
                        Some("runtime entry is called without arguments"),
                    )
                    .with_help("remove the parameters or rename this function")
            })
        })
        .collect()
}

/// Returns whether the compiler must synthesize this contract's runtime entry.
///
/// Any contract-local ordinary function named `main` is a
/// user-supplied runtime entry, irrespective of visibility.
pub fn contract_needs_generated_dispatch<'db>(db: &'db dyn Db, contract: ContractDef<'db>) -> bool {
    !contract.has_runtime_main(db)
}

fn contract_dispatch_surface_with_resolutions<'db>(
    db: &'db dyn Db,
    module: Module<'db>,
    item_resolutions: &hir_nameres::ItemResolutionFacts<'db>,
    contract: ContractDef<'db>,
) -> DispatchSurface<'db> {
    let contract_name = ident_text(db, &contract.name_elem(db));
    let abi_evidence = visible_abi_adt_evidence(db, module);
    let contract_type_vars =
        type_var_bindings(contract.def_id_value(db), contract.ty_param_elems(db));
    let mut diagnostics = Vec::new();
    let mut methods = Vec::new();
    let mut constructor: Option<DispatchConstructor> = None;
    let mut constructor_abi_diagnostics = Vec::new();
    let mut fallback: Option<DispatchFallback<'db>> = None;

    for (source_index, item) in contract.items(db).iter().enumerate() {
        let ContractItem::FunctionDef(function) = *item else {
            continue;
        };
        match function.kind(db) {
            FuncKind::Function => {
                let sig = function.sig(db);
                if sig.public.is_none() || ident_text(db, &sig.name) == "fallback" {
                    continue;
                }
                let type_vars =
                    function_type_vars(db, &contract_type_vars, function.def_id_value(db), sig);
                let lowered = lower_normalized_function(
                    db,
                    module,
                    item_resolutions,
                    contract.def_id_value(db),
                    function,
                    &type_vars,
                );
                let param_names = param_names(db, sig.params.atom());
                let inputs = abi_params(
                    db,
                    &param_names,
                    &lowered.params,
                    &mut diagnostics,
                    sig.span,
                    &abi_evidence,
                );
                let outputs =
                    abi_outputs(db, lowered.ret, &mut diagnostics, sig.span, &abi_evidence);
                let signature = method_signature_string(
                    db,
                    &ident_text(db, &sig.name),
                    &lowered.params,
                    &abi_evidence,
                )
                .unwrap_or_else(|err| {
                    diagnostics.push(contract_diag_unsupported_abi_type(
                        db,
                        sig.span,
                        &ident_text(db, &sig.name),
                        &err,
                    ));
                    format!("{}(<unsupported>)", ident_text(db, &sig.name))
                });
                let selector = abi_selector(db, AbiSignature::new(db, signature.clone()));
                methods.push(DispatchMethod {
                    def: function.def_id_value(db),
                    source_index,
                    name: ident_text(db, &sig.name),
                    payable: sig.payable.is_some(),
                    signature,
                    selector,
                    inputs,
                    outputs,
                });
            }
            FuncKind::Constructor => {
                if constructor.is_some() {
                    diagnostics.push(contract_diag_multiple_constructors(db, function.span(db)));
                    continue;
                }
                let sig = function.sig(db);
                let type_vars =
                    function_type_vars(db, &contract_type_vars, function.def_id_value(db), sig);
                let lowered = lower_normalized_function(
                    db,
                    module,
                    item_resolutions,
                    contract.def_id_value(db),
                    function,
                    &type_vars,
                );
                let inputs = abi_params(
                    db,
                    &param_names(db, sig.params.atom()),
                    &lowered.params,
                    &mut constructor_abi_diagnostics,
                    sig.span,
                    &abi_evidence,
                );
                diagnostics.extend(constructor_abi_diagnostics.iter().cloned());
                constructor = Some(DispatchConstructor::Explicit {
                    source_index,
                    payable: sig.payable.is_some(),
                    inputs,
                });
            }
            FuncKind::Fallback => {
                if fallback.is_some() {
                    diagnostics.push(contract_diag_multiple_fallbacks(db, function.span(db)));
                    continue;
                }
                let sig = function.sig(db);
                let type_vars =
                    function_type_vars(db, &contract_type_vars, function.def_id_value(db), sig);
                let lowered = lower_normalized_function(
                    db,
                    module,
                    item_resolutions,
                    contract.def_id_value(db),
                    function,
                    &type_vars,
                );
                let inputs = abi_params(
                    db,
                    &param_names(db, sig.params.atom()),
                    &lowered.params,
                    &mut diagnostics,
                    sig.span,
                    &abi_evidence,
                );
                let outputs =
                    abi_outputs(db, lowered.ret, &mut diagnostics, sig.span, &abi_evidence);
                if !inputs.is_empty() || !outputs.is_empty() {
                    diagnostics.push(contract_diag_unsupported_fallback_shape(
                        db,
                        function.span(db),
                    ));
                }
                fallback = Some(DispatchFallback::Explicit {
                    def: function.def_id_value(db),
                    source_index,
                    payable: sig.payable.is_some(),
                    inputs,
                    outputs,
                });
            }
        }
    }

    let constructor = constructor.unwrap_or(DispatchConstructor::Implicit);
    let fallback = fallback.unwrap_or(DispatchFallback::Default);

    let mut seen_signatures = FxHashMap::<String, usize>::default();
    let mut seen_selectors = FxHashMap::<AbiSelector, usize>::default();
    for (method_index, method) in methods.iter().enumerate() {
        if abi_params_contain_unsupported(&method.inputs) {
            continue;
        }

        if let Some(&previous_index) = seen_signatures.get(&method.signature) {
            diagnostics.push(contract_diag_duplicate_signature(
                db,
                dispatch_method_span(db, contract, method),
                dispatch_method_span(db, contract, &methods[previous_index]),
                &contract_name,
                &method.signature,
            ));
        } else {
            seen_signatures.insert(method.signature.clone(), method_index);
        }

        if let Some(&previous_index) = seen_selectors.get(&method.selector) {
            let previous = &methods[previous_index];
            if previous.signature != method.signature {
                diagnostics.push(contract_diag_selector_collision(
                    db,
                    dispatch_method_span(db, contract, method),
                    dispatch_method_span(db, contract, previous),
                    &contract_name,
                    method,
                    previous,
                ));
            }
        } else {
            seen_selectors.insert(method.selector, method_index);
        }
    }

    DispatchSurface {
        contract: contract.def_id_value(db),
        name: contract_name,
        methods,
        constructor,
        fallback,
        constructor_abi_diagnostics,
        diagnostics,
    }
}

fn abi_params_contain_unsupported(params: &[AbiParam]) -> bool {
    params.iter().any(|param| {
        matches!(&param.ty, AbiType::Unsupported)
            || abi_params_contain_unsupported(&param.components)
    })
}

fn contract_diag_duplicate_signature<'db>(
    db: &'db dyn Db,
    current_span: hir::span::Span<'db>,
    previous_span: hir::span::Span<'db>,
    contract: &str,
    signature: &str,
) -> Diagnostic {
    Diagnostic::error(format!(
        "duplicate public ABI signature in contract `{contract}`: {signature}"
    ))
    .with_code("SC0230")
    .with_primary_label(db, current_span, Some("duplicate ABI signature"))
    .with_secondary_label(db, previous_span, Some("previous declaration"))
}

fn contract_diag_selector_collision<'db>(
    db: &'db dyn Db,
    current_span: hir::span::Span<'db>,
    previous_span: hir::span::Span<'db>,
    contract: &str,
    current: &DispatchMethod<'db>,
    previous: &DispatchMethod<'db>,
) -> Diagnostic {
    Diagnostic::error(format!(
        "public ABI selector collision in contract `{contract}`: `{}` and `{}` both use {}",
        previous.signature,
        current.signature,
        current.selector.to_hex(),
    ))
    .with_code(DiagnosticCode::TYPECK_CONTRACT_SELECTOR_COLLISION)
    .with_primary_label(
        db,
        current_span,
        Some(format!("`{}` collides here", current.signature)),
    )
    .with_secondary_label(
        db,
        previous_span,
        Some(format!("`{}` first used this selector", previous.signature)),
    )
}

fn dispatch_method_span<'db>(
    db: &'db dyn Db,
    contract: ContractDef<'db>,
    method: &DispatchMethod<'db>,
) -> hir::span::Span<'db> {
    match contract.items(db).get(method.source_index) {
        Some(ContractItem::FunctionDef(function)) => function.sig(db).span,
        _ => contract.name_elem(db).span(db),
    }
}

fn contract_diag_multiple_constructors<'db>(
    db: &'db dyn Db,
    span: hir::span::Span<'db>,
) -> Diagnostic {
    Diagnostic::error("contract has more than one constructor")
        .with_code("SC0232")
        .with_primary_label(db, span, Some("extra constructor"))
}

fn contract_diag_multiple_fallbacks<'db>(
    db: &'db dyn Db,
    span: hir::span::Span<'db>,
) -> Diagnostic {
    Diagnostic::error("contract has more than one fallback")
        .with_code("SC0233")
        .with_primary_label(db, span, Some("extra fallback"))
}

fn contract_diag_unsupported_fallback_shape<'db>(
    db: &'db dyn Db,
    span: hir::span::Span<'db>,
) -> Diagnostic {
    Diagnostic::error("fallback ABI must be unit -> unit")
        .with_code("SC0231")
        .with_primary_label(db, span, Some("unsupported fallback ABI"))
}
