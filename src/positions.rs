//! Sealed position-marker traits enforcing the [RDF 1.1 Concepts §3 Triples][s3]
//! and [§4 Datasets][s4] abstract syntax.
//!
//! | term kind  | subject | predicate | object | graph |
//! | ---------- | :-----: | :-------: | :----: | :---: |
//! | IRI        |    ✓    |     ✓     |   ✓    |   ✓   |
//! | blank node |    ✓    |     —     |   ✓    |   ✓   |
//! | literal    |    —    |     —     |   ✓    |   —   |
//!
//! [`IsSubject`], [`IsPredicate`], [`IsObject`], [`IsGraph`] are sealed:
//! only this crate (for lexical RDF types) and types declared via
//! [`impl_resource!`] (the unsealed [`Resource`] marker) may satisfy them.
//!
//! Generalized RDF (RDF 1.1 §A) — literals as subjects, blank nodes as
//! predicates — is opt-in via the parallel `Generalized*` types, which place
//! no position bounds.
//!
//! [s3]: https://www.w3.org/TR/rdf11-concepts/#section-triples
//! [s4]: https://www.w3.org/TR/rdf11-concepts/#section-dataset

use std::borrow::Cow;

use iri_rs::{Iri, IriBuf};

use crate::{
    BlankId,
    BlankIdBuf,
    CowId,
    CowLiteral,
    CowLocalTerm,
    CowTerm,
    Id,
    IdRef,
    Literal,
    LiteralRef,
    LocalTerm,
    LocalTermRef,
    Term,
    TermRef,
};

#[doc(hidden)]
pub mod sealed {
    pub trait Sealed {}
}

/// Types that may appear in the subject position of an RDF triple/quad.
pub trait IsSubject: sealed::Sealed {}
/// Types that may appear in the predicate position of an RDF triple/quad.
pub trait IsPredicate: sealed::Sealed {}
/// Types that may appear in the object position of an RDF triple/quad.
pub trait IsObject: sealed::Sealed {}
/// Types that may appear in the graph-name position of an RDF quad.
pub trait IsGraph: sealed::Sealed {}

/// Marker trait for opaque resource handles (e.g. interpretation indices).
///
/// Implement via [`impl_resource!`], which also emits the four position-trait
/// impls so the type is usable in every position. The supertraits make
/// `R: Resource` propagate position-trait bounds automatically — concrete
/// `Resource` impls must also satisfy [`IsSubject`], [`IsPredicate`],
/// [`IsObject`], and [`IsGraph`], which `impl_resource!` handles.
pub trait Resource: IsSubject + IsPredicate + IsObject + IsGraph {}

impl<T: sealed::Sealed + ?Sized> sealed::Sealed for &T {}
impl<T: IsSubject + ?Sized> IsSubject for &T {}
impl<T: IsPredicate + ?Sized> IsPredicate for &T {}
impl<T: IsObject + ?Sized> IsObject for &T {}
impl<T: IsGraph + ?Sized> IsGraph for &T {}
impl<T: Resource + ?Sized> Resource for &T {}

// Option-of-position is also a position — used by canonical pattern types
// where `None` denotes "any".
impl<T: sealed::Sealed> sealed::Sealed for Option<T> {}
impl<T: IsSubject> IsSubject for Option<T> {}
impl<T: IsPredicate> IsPredicate for Option<T> {}
impl<T: IsObject> IsObject for Option<T> {}
impl<T: IsGraph> IsGraph for Option<T> {}

macro_rules! impl_iri_positions {
    ($t:ty) => {
        impl sealed::Sealed for $t {}
        impl IsSubject for $t {}
        impl IsPredicate for $t {}
        impl IsObject for $t {}
        impl IsGraph for $t {}
    };
}

impl_iri_positions!(IriBuf);
impl_iri_positions!(Iri<&'_ str>);
impl_iri_positions!(Cow<'_, IriBuf>);

macro_rules! impl_id_positions {
    ($t:ty) => {
        impl sealed::Sealed for $t {}
        impl IsSubject for $t {}
        impl IsObject for $t {}
        impl IsGraph for $t {}
    };
}

impl_id_positions!(BlankIdBuf);
impl_id_positions!(BlankId);
impl_id_positions!(Cow<'_, BlankId>);
impl_id_positions!(Id);
impl_id_positions!(IdRef<'_>);
impl_id_positions!(CowId<'_>);

macro_rules! impl_obj_positions {
    ($t:ty) => {
        impl sealed::Sealed for $t {}
        impl IsObject for $t {}
    };
}

impl_obj_positions!(Literal);
impl_obj_positions!(LiteralRef<'_>);
impl_obj_positions!(CowLiteral<'_>);
impl_obj_positions!(Term);
impl_obj_positions!(TermRef<'_>);
impl_obj_positions!(CowTerm<'_>);
impl_obj_positions!(LocalTerm);
impl_obj_positions!(LocalTermRef<'_>);
impl_obj_positions!(CowLocalTerm<'_>);

/// Implements [`Resource`] and the four position traits for a type, granting
/// it eligibility in every position of a triple/quad.
#[macro_export]
macro_rules! impl_resource {
    ($t:ty) => {
        impl $crate::Resource for $t {}
        impl $crate::sealed::Sealed for $t {}
        impl $crate::IsSubject for $t {}
        impl $crate::IsPredicate for $t {}
        impl $crate::IsObject for $t {}
        impl $crate::IsGraph for $t {}
    };
}

impl_resource!(usize);
impl_resource!(u32);
impl_resource!(u64);
impl_resource!(i32);
impl_resource!(i64);
impl_resource!(String);

impl Resource for str {}
impl sealed::Sealed for str {}
impl IsSubject for str {}
impl IsPredicate for str {}
impl IsObject for str {}
impl IsGraph for str {}
