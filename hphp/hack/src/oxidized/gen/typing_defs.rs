// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the "hack" directory of this source tree.
//
// @generated SignedSource<<9d42de83572fc85fb656c1d7c3581a65>>
//
// To regenerate this file, run:
//   hphp/hack/src/oxidized_regen.sh

use eq_modulo_pos::EqModuloPos;
use no_pos_hash::NoPosHash;
use ocamlrep_derive::FromOcamlRep;
use ocamlrep_derive::FromOcamlRepIn;
use ocamlrep_derive::ToOcamlRep;
use serde::Deserialize;
use serde::Serialize;

#[allow(unused_imports)]
use crate::*;

pub use typing_defs_flags::*;

pub use typing_defs_core::*;

/// Origin of Class Constant References:
/// In order to be able to detect cycle definitions like
/// class C {
/// const int A = D::A;
/// }
/// class D {
/// const int A = C::A;
/// }
/// we need to remember which constants were used during initialization.
///
/// Currently the syntax of constants allows direct references to another class
/// like D::A, or self references using self::A.
///
/// class_const_from encodes the origin (class vs self).
#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C, u8)]
pub enum ClassConstFrom {
    Self_,
    From(String),
}

/// Class Constant References:
/// In order to be able to detect cycle definitions like
/// class C {
/// const int A = D::A;
/// }
/// class D {
/// const int A = C::A;
/// }
/// we need to remember which constants were used during initialization.
///
/// Currently the syntax of constants allows direct references to another class
/// like D::A, or self references using self::A.
#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct ClassConstRef(pub ClassConstFrom, pub String);

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct ConstDecl {
    pub pos: pos_or_decl::PosOrDecl,
    pub type_: Ty,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct ClassElt {
    pub visibility: CeVisibility,
    pub type_: lazy::Lazy<Ty>,
    /// identifies the class from which this elt originates
    pub origin: String,
    pub deprecated: Option<String>,
    /// pos of the type of the elt
    pub pos: lazy::Lazy<pos_or_decl::PosOrDecl>,
    pub flags: typing_defs_flags::class_elt::ClassElt,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct FunElt {
    pub deprecated: Option<String>,
    pub module: Option<ast_defs::Id>,
    /// Top-level functions have limited visibilities
    pub internal: bool,
    pub type_: Ty,
    pub pos: pos_or_decl::PosOrDecl,
    pub php_std_lib: bool,
    pub support_dynamic_type: bool,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    FromOcamlRepIn,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C, u8)]
pub enum ClassConstKind {
    CCAbstract(bool),
    CCConcrete,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct ClassConst {
    pub synthesized: bool,
    pub abstract_: ClassConstKind,
    pub pos: pos_or_decl::PosOrDecl,
    pub type_: Ty,
    /// identifies the class from which this const originates
    pub origin: String,
    /// references to the constants used in the initializer
    pub refs: Vec<ClassConstRef>,
}

/// The position is that of the hint in the `use` / `implements` AST node
/// that causes a class to have this requirement applied to it. E.g.
///
/// ```
/// class Foo {}
///
/// interface Bar {
///   require extends Foo; <- position of the decl_phase ty
/// }
///
/// class Baz extends Foo implements Bar { <- position of the `implements`
/// }
/// ```
#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct Requirement(pub pos_or_decl::PosOrDecl, pub Ty);

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct AbstractTypeconst {
    pub as_constraint: Option<Ty>,
    pub super_constraint: Option<Ty>,
    pub default: Option<Ty>,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct ConcreteTypeconst {
    pub tc_type: Ty,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct PartiallyAbstractTypeconst {
    pub constraint: Ty,
    pub type_: Ty,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C, u8)]
pub enum Typeconst {
    TCAbstract(AbstractTypeconst),
    TCConcrete(ConcreteTypeconst),
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct TypeconstType {
    pub synthesized: bool,
    pub name: PosId,
    pub kind: Typeconst,
    pub origin: String,
    /// If the typeconst had the <<__Enforceable>> attribute on its
    /// declaration, this will be [(position_of_declaration, true)].
    ///
    /// In legacy decl, the second element of the tuple will also be true if
    /// the typeconst overrides some parent typeconst which had the
    /// <<__Enforceable>> attribute. In that case, the position will point to
    /// the declaration of the parent typeconst.
    ///
    /// In shallow decl, this is not the case--there is no overriding behavior
    /// modeled here, and the second element will only be true when the
    /// declaration of this typeconst had the attribute.
    ///
    /// When the second element of the tuple is false, the position will be
    /// [Pos_or_decl.none].
    ///
    /// To manage the difference between legacy and shallow decl, use
    /// [Typing_classes_heap.Api.get_typeconst_enforceability] rather than
    /// accessing this field directly.
    pub enforceable: (pos_or_decl::PosOrDecl, bool),
    pub reifiable: Option<pos_or_decl::PosOrDecl>,
    pub concretized: bool,
    pub is_ctx: bool,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct EnumType {
    pub base: Ty,
    pub constraint: Option<Ty>,
    pub includes: Vec<Ty>,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C)]
pub struct TypedefType {
    pub module: Option<ast_defs::Id>,
    pub pos: pos_or_decl::PosOrDecl,
    pub vis: aast::TypedefVisibility,
    pub tparams: Vec<Tparam>,
    pub constraint: Option<Ty>,
    pub type_: Ty,
    pub is_ctx: bool,
    pub attributes: Vec<UserAttribute>,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    EqModuloPos,
    FromOcamlRep,
    Hash,
    NoPosHash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    ToOcamlRep
)]
#[repr(C, u8)]
pub enum DeserializationError {
    /// The type was valid, but some component thereof was a decl_ty when we
    /// expected a locl_phase ty, or vice versa.
    WrongPhase(String),
    /// The specific type or some component thereof is not one that we support
    /// deserializing, usually because not enough information was serialized to be
    /// able to deserialize it again.
    NotSupported(String),
    /// The input JSON was invalid for some reason.
    DeserializationError(String),
}
