//! Canonical standard-library definitions used by dispatch and ABI checking.

use hir::{
    anchor::{DefId, DefKind},
    ast::item::Item,
};
use nameres::{LibraryId, ModuleId, module_id_for_source_file};
use parser::parse_file_to_hir;

use crate::{Db, Ty, TyCtor, UserTyCtor, UserTyCtorKind, support::canonical_std_module_ids};

pub(super) const ABI_HEAD_SIZE_METHOD: &str = "headSize";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StdAbiModule {
    Std,
    Dispatch,
    Generic,
}

impl StdAbiModule {
    fn name(self) -> &'static str {
        match self {
            Self::Std => "std",
            Self::Dispatch => "dispatch",
            Self::Generic => "Generic",
        }
    }

    fn candidates<'db>(self, db: &'db dyn Db) -> impl Iterator<Item = ModuleId<'db>> {
        canonical_std_module_ids(db, self.name())
    }

    /// Uses the standard-library identity first, including when its sources
    /// are also available under the main module root.
    pub(crate) fn module<'db>(self, db: &'db dyn Db) -> Option<ModuleId<'db>> {
        self.candidates(db)
            .find(|module| db.module_file(*module).is_some())
    }

    fn contains(self, db: &dyn Db, def: DefId<'_>) -> bool {
        module_id_for_source_file(db, def.file(db)).is_some_and(|module| {
            module.library(db) == &LibraryId::Std
                && module.logical_path(db).as_slice() == [self.name()]
        })
    }

    fn find_def<'db>(self, db: &'db dyn Db, name: &str, kind: DefKind) -> Option<DefId<'db>> {
        self.candidates(db).find_map(|module| {
            let file = db.module_file(module)?;
            parse_file_to_hir(db, file)
                .module(db)
                .items(db)
                .iter()
                .find_map(|item| {
                    let def = match item {
                        Item::AdtDef(adt) => adt.def_id_value(db),
                        Item::ClassDef(class) => class.def_id_value(db),
                        Item::FunctionDef(function) => function.def_id_value(db),
                        _ => return None,
                    };
                    (def.kind(db) == kind && def.name(db).as_deref() == Some(name)).then_some(def)
                })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum AbiClass {
    Generic,
    SigString,
    Attribs,
    Encode,
    Decode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum AbiEvidenceRole {
    Signature,
    Codec,
}

impl AbiClass {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Generic => "Generic",
            Self::SigString => "SigString",
            Self::Attribs => "ABIAttribs",
            Self::Encode => "ABIEncode",
            Self::Decode => "ABIDecode",
        }
    }

    fn module(self) -> StdAbiModule {
        match self {
            Self::Generic => StdAbiModule::Generic,
            Self::SigString => StdAbiModule::Dispatch,
            Self::Attribs | Self::Encode | Self::Decode => StdAbiModule::Std,
        }
    }

    pub(crate) fn evidence_role(self) -> Option<AbiEvidenceRole> {
        match self {
            Self::Generic => None,
            Self::SigString => Some(AbiEvidenceRole::Signature),
            Self::Attribs | Self::Encode | Self::Decode => Some(AbiEvidenceRole::Codec),
        }
    }

    pub(crate) fn from_def(db: &dyn Db, def: DefId<'_>) -> Option<Self> {
        let name = def.name(db)?;
        [
            Self::Generic,
            Self::SigString,
            Self::Attribs,
            Self::Encode,
            Self::Decode,
        ]
        .into_iter()
        .find(|class| class.name() == name && class.module().contains(db, def))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StdAbiType {
    CalldataWordReader,
    MemoryWordReader,
    Method,
    Payable,
    NonPayable,
    Proxy,
    Memory,
    Calldata,
    Bytes,
}

impl StdAbiType {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::CalldataWordReader => "CalldataWordReader",
            Self::MemoryWordReader => "MemoryWordReader",
            Self::Method => "Method",
            Self::Payable => "Payable",
            Self::NonPayable => "NonPayable",
            Self::Proxy => "Proxy",
            Self::Memory => "memory",
            Self::Calldata => "calldata",
            Self::Bytes => "bytes",
        }
    }

    fn module(self) -> StdAbiModule {
        match self {
            Self::Method | Self::Payable | Self::NonPayable => StdAbiModule::Dispatch,
            Self::CalldataWordReader
            | Self::MemoryWordReader
            | Self::Proxy
            | Self::Memory
            | Self::Calldata
            | Self::Bytes => StdAbiModule::Std,
        }
    }

    pub(crate) fn def<'db>(self, db: &'db dyn Db) -> Option<DefId<'db>> {
        self.module().find_def(db, self.name(), DefKind::Adt)
    }

    pub(crate) fn ty<'db>(self, db: &'db dyn Db, args: Vec<Ty<'db>>) -> Option<Ty<'db>> {
        self.def(db).map(|def| {
            Ty::named(
                db,
                TyCtor::User(UserTyCtor {
                    def,
                    kind: UserTyCtorKind::Adt,
                }),
                args,
            )
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DispatchClass {
    Selector,
    ExecMethod,
}

impl DispatchClass {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Selector => "Selector",
            Self::ExecMethod => "ExecMethod",
        }
    }

    pub(crate) fn from_def(db: &dyn Db, def: DefId<'_>) -> Option<Self> {
        let name = def.name(db)?;
        [Self::Selector, Self::ExecMethod]
            .into_iter()
            .find(|class| class.name() == name && StdAbiModule::Dispatch.contains(db, def))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StdAbiFunction {
    Decode,
    Encode,
}

impl StdAbiFunction {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Decode => "abi_decode",
            Self::Encode => "abi_encode",
        }
    }

    pub(crate) fn def<'db>(self, db: &'db dyn Db) -> Option<DefId<'db>> {
        StdAbiModule::Std.find_def(db, self.name(), DefKind::Function)
    }
}
