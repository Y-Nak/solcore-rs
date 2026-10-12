//! ABI obligations of the standard-library operations emitted by preparation.
//!
//! Read the declared instance/function contexts rather than maintaining a
//! second list of ABI constraints beside `std.dispatch` and `std.abi_decode`.
//! Each ABI condition is still solved separately: an unrelated failed call or
//! missing dictionary must not hide a selected manual ABI implementation.

use hir::anchor::DefId;
use nameres::module_id_for_source_file;
use rustc_hash::FxHashMap;

use super::std_abi::{
    ABI_HEAD_SIZE_METHOD, AbiClass, DispatchClass, StdAbiFunction, StdAbiModule, StdAbiType,
};
use crate::{
    ClassId, ClauseOrigin, Db, Pred, PredKind, Ty, TyKind, function_scheme,
    infer::class_method_scheme,
    solver::{
        AbiEvidenceRole, clause_conditions_for_goal, scheme_conditions_for_type,
        trait_env_for_module,
    },
};

/// The argument order shared by generated `Method` values and their types.
pub(crate) struct MethodParts<T> {
    pub name: T,
    pub payability: T,
    pub args: T,
    pub returns: T,
    pub implementation: T,
}

impl<T> MethodParts<T> {
    pub fn into_args(self) -> Vec<T> {
        vec![
            self.name,
            self.payability,
            self.args,
            self.returns,
            self.implementation,
        ]
    }
}

pub(super) fn method_abi_roots<'db>(
    db: &'db dyn Db,
    classes: &FxHashMap<AbiClass, DefId<'db>>,
    method: MethodParts<Ty<'db>>,
) -> Vec<(AbiEvidenceRole, Pred<'db>)> {
    let args = method.args;
    let Some(module_id) = StdAbiModule::Dispatch.module(db) else {
        // Missing dispatch imports still receive their normal resolution errors.
        // Preserve independent ABI diagnostics using the std codec operations
        // called by do_exec, without implicitly importing dispatch for the user.
        return method_codec_abi_roots(db, classes, args, method.returns);
    };
    let Some(method_ty) = StdAbiType::Method.ty(db, method.into_args()) else {
        return Vec::new();
    };
    let env = trait_env_for_module(db, module_id);
    let mut roots = Vec::new();
    for clause in env.clauses(db) {
        let ClauseOrigin::Instance { def, .. } = clause.origin else {
            continue;
        };
        // Only the std declarations describe the generated dispatch protocol.
        if module_id_for_source_file(db, def.file(db)) != Some(module_id) {
            continue;
        }
        let PredKind::InClass {
            class: ClassId::User(class),
            ..
        } = clause.head.kind(db)
        else {
            continue;
        };
        if DispatchClass::from_def(db, *class).is_none() {
            continue;
        }
        let goal = Pred::in_class(db, ClassId::User(*class), method_ty, Vec::new());
        if let Some(conditions) = clause_conditions_for_goal(db, &clause, goal) {
            roots.extend(abi_roots(db, classes, conditions));
        }
    }
    sort_abi_roots(db, args, &mut roots);
    roots
}

fn sort_abi_roots<'db>(db: &'db dyn Db, args: Ty<'db>, roots: &mut [(AbiEvidenceRole, Pred<'db>)]) {
    // Preserve the traversal order, including the first exhausted proof and
    // the shared derived-evidence budget, independently of declaration order.
    roots.sort_by_key(|(_, pred)| {
        let PredKind::InClass {
            class: ClassId::User(class),
            main,
            ..
        } = pred.kind(db)
        else {
            unreachable!("ABI roots are canonical class predicates")
        };
        let order = match AbiClass::from_def(db, *class).expect("canonical ABI root") {
            AbiClass::SigString => 0,
            AbiClass::Attribs => 1,
            AbiClass::Encode => 2,
            AbiClass::Decode => 3,
            AbiClass::Generic => unreachable!("Generic is checked by exposed ADT identity"),
        };
        (order, *main != args)
    });
}

pub(super) fn constructor_abi_roots<'db>(
    db: &'db dyn Db,
    classes: &FxHashMap<AbiClass, DefId<'db>>,
    args: Ty<'db>,
) -> Vec<(AbiEvidenceRole, Pred<'db>)> {
    decode_abi_roots(
        db,
        classes,
        args,
        StdAbiType::Memory,
        StdAbiType::MemoryWordReader,
    )
}

fn method_codec_abi_roots<'db>(
    db: &'db dyn Db,
    classes: &FxHashMap<AbiClass, DefId<'db>>,
    args: Ty<'db>,
    returns: Ty<'db>,
) -> Vec<(AbiEvidenceRole, Pred<'db>)> {
    let mut roots = Vec::new();
    if let Some(module_id) = StdAbiModule::Std.module(db)
        && let Some(class) = classes.get(&AbiClass::Attribs)
        && let Some(scheme) =
            class_method_scheme(db, module_id, *class, ABI_HEAD_SIZE_METHOD.into())
        && let Some(proxy) = StdAbiType::Proxy.ty(db, vec![args])
        && let Some(conditions) =
            scheme_conditions_for_type(db, scheme, Ty::function(db, vec![proxy], Ty::word(db)))
    {
        roots.extend(abi_roots(db, classes, conditions));
    }
    if let Some(bytes) = StdAbiType::Bytes.ty(db, Vec::new())
        && let Some(output) = StdAbiType::Memory.ty(db, vec![bytes])
    {
        roots.extend(function_abi_roots(
            db,
            classes,
            StdAbiFunction::Encode,
            Ty::function(db, vec![returns], output),
        ));
    }
    roots.extend(decode_abi_roots(
        db,
        classes,
        args,
        StdAbiType::Calldata,
        StdAbiType::CalldataWordReader,
    ));
    sort_abi_roots(db, args, &mut roots);
    roots
}

fn decode_abi_roots<'db>(
    db: &'db dyn Db,
    classes: &FxHashMap<AbiClass, DefId<'db>>,
    args: Ty<'db>,
    location: StdAbiType,
    reader: StdAbiType,
) -> Vec<(AbiEvidenceRole, Pred<'db>)> {
    let roots = || {
        let source = location.ty(db, vec![StdAbiType::Bytes.ty(db, Vec::new())?])?;
        let args_proxy = StdAbiType::Proxy.ty(db, vec![args])?;
        let reader = reader.ty(db, Vec::new())?;
        let reader_proxy = StdAbiType::Proxy.ty(db, vec![reader])?;
        let call_ty = Ty::function(db, vec![source, args_proxy, reader_proxy], args);
        Some(function_abi_roots(
            db,
            classes,
            StdAbiFunction::Decode,
            call_ty,
        ))
    };
    roots().unwrap_or_default()
}

fn function_abi_roots<'db>(
    db: &'db dyn Db,
    classes: &FxHashMap<AbiClass, DefId<'db>>,
    function: StdAbiFunction,
    call_ty: Ty<'db>,
) -> Vec<(AbiEvidenceRole, Pred<'db>)> {
    let roots = || {
        let module_id = StdAbiModule::Std.module(db)?;
        let function = function.def(db)?;
        let scheme = function_scheme(db, module_id, function)?;
        let conditions = scheme_conditions_for_type(db, scheme, call_ty)?;
        Some(abi_roots(db, classes, conditions))
    };
    roots().unwrap_or_default()
}

fn abi_roots<'db>(
    db: &'db dyn Db,
    classes: &FxHashMap<AbiClass, DefId<'db>>,
    conditions: Vec<Pred<'db>>,
) -> Vec<(AbiEvidenceRole, Pred<'db>)> {
    conditions
        .into_iter()
        .filter_map(|pred| {
            let PredKind::InClass {
                class: ClassId::User(class),
                main,
                ..
            } = pred.kind(db)
            else {
                return None;
            };
            let kind = AbiClass::from_def(db, *class)?;
            // An absent generated name has no signature obligation. Other roots
            // remain inspectable even when preparation could not build the name.
            if (kind == AbiClass::SigString && matches!(main.kind(db), TyKind::Unknown))
                || classes.get(&kind) != Some(class)
            {
                return None;
            }
            Some((kind.evidence_role()?, pred))
        })
        .collect()
}
