//! Metadata-tagged terms and triples/quads, gated on the `meta` feature.
//!
//! Provides a generic [`Meta<T, M>`] wrapper that pairs any value with metadata
//! (e.g. a [`locspan::Span`] or [`locspan::Location`]). The [`Strip`] trait
//! discards the metadata layer.
//!
//! # Examples
//!
//! Round-tripping a tagged value through [`Strip`]:
//!
//! ```
//! use rdf_rs::meta::{Meta, Span, Strip};
//!
//! let tagged = Meta(42usize, Span::new(0, 5));
//! assert_eq!(tagged.strip(), 42usize);
//! ```
//!
//! Using [`Meta`] as a dataset key to keep otherwise-equal quads distinct
//! by their metadata:
//!
//! ```
//! use rdf_rs::{Quad, dataset::BTreeDataset, meta::{Meta, Span}};
//!
//! let s: usize = 0;
//! let p: usize = 1;
//! let o: usize = 2;
//!
//! let mut d: BTreeDataset<Meta<usize, Span>> = BTreeDataset::new();
//! d.insert(Quad(Meta(s, Span::new(0, 1)), Meta(p, Span::new(0, 1)), Meta(o, Span::new(0, 1)), None));
//! d.insert(Quad(Meta(s, Span::new(2, 3)), Meta(p, Span::new(2, 3)), Meta(o, Span::new(2, 3)), None));
//! assert_eq!(d.len(), 2);
//! ```

use crate::{BlankIdBuf, Datatype, Id, IsGraph, IsObject, IsPredicate, IsSubject, Literal, LiteralType, LocalTerm, Quad, Resource, Term, Triple};
use iri_rs::IriBuf;

#[doc(no_inline)]
pub use locspan::{Location, Span};

/// A value of type `T` annotated with metadata `M`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Meta<T, M>(pub T, pub M);

impl<T: crate::sealed::Sealed, M> crate::sealed::Sealed for Meta<T, M> {}
impl<T: IsSubject, M> IsSubject for Meta<T, M> {}
impl<T: IsPredicate, M> IsPredicate for Meta<T, M> {}
impl<T: IsObject, M> IsObject for Meta<T, M> {}
impl<T: IsGraph, M> IsGraph for Meta<T, M> {}
impl<T: Resource, M> Resource for Meta<T, M> {}

impl<T, M> Meta<T, M> {
    pub const fn new(value: T, metadata: M) -> Self {
        Self(value, metadata)
    }

    pub const fn value(&self) -> &T {
        &self.0
    }

    pub const fn metadata(&self) -> &M {
        &self.1
    }

    pub fn into_value(self) -> T {
        self.0
    }

    pub fn into_metadata(self) -> M {
        self.1
    }

    pub fn into_parts(self) -> (T, M) {
        (self.0, self.1)
    }
}

/// Discards metadata layers.
pub trait Strip {
    type Stripped;

    fn strip(self) -> Self::Stripped;
}

impl<T: Strip, M> Strip for Meta<T, M> {
    type Stripped = T::Stripped;

    fn strip(self) -> Self::Stripped {
        self.0.strip()
    }
}

macro_rules! identity_strip {
    ($($t:ty),* $(,)?) => {
        $(
            impl Strip for $t {
                type Stripped = $t;

                fn strip(self) -> Self::Stripped {
                    self
                }
            }
        )*
    };
}

identity_strip!(BlankIdBuf, IriBuf, Id, Datatype, Literal, LiteralType, Term, LocalTerm, usize, u32, u64, i32, i64, String);

impl<S, P, O> Strip for Triple<S, P, O>
where
    S: IsSubject + Strip,
    P: IsPredicate + Strip,
    O: IsObject + Strip,
    S::Stripped: IsSubject,
    P::Stripped: IsPredicate,
    O::Stripped: IsObject,
{
    type Stripped = Triple<S::Stripped, P::Stripped, O::Stripped>;

    fn strip(self) -> Self::Stripped {
        Triple(self.0.strip(), self.1.strip(), self.2.strip())
    }
}

impl<S, P, O, G> Strip for Quad<S, P, O, G>
where
    S: IsSubject + Strip,
    P: IsPredicate + Strip,
    O: IsObject + Strip,
    G: IsGraph + Strip,
    S::Stripped: IsSubject,
    P::Stripped: IsPredicate,
    O::Stripped: IsObject,
    G::Stripped: IsGraph,
{
    type Stripped = Quad<S::Stripped, P::Stripped, O::Stripped, G::Stripped>;

    fn strip(self) -> Self::Stripped {
        Quad(self.0.strip(), self.1.strip(), self.2.strip(), self.3.map(Strip::strip))
    }
}

/// `Term` with metadata.
pub type MetaTerm<M> = Meta<Term, M>;
/// `LocalTerm` with metadata.
pub type MetaLocalTerm<M> = Meta<LocalTerm, M>;
/// `Id` with metadata.
pub type MetaId<M> = Meta<Id, M>;
/// `Literal` with metadata.
pub type MetaLiteral<M> = Meta<Literal, M>;

/// Triple of `LocalTerm` components, all sharing the same metadata `M`,
/// itself wrapped in metadata.
pub type MetaTriple<M> = Meta<Triple<MetaLocalTerm<M>>, M>;
/// Quad of `LocalTerm` components, all sharing the same metadata `M`,
/// itself wrapped in metadata.
pub type MetaQuad<M> = Meta<Quad<MetaLocalTerm<M>>, M>;

/// Triple with heterogeneous component types, each wrapped in metadata `M`,
/// the whole triple itself wrapped in metadata.
pub type MetaTripleFull<S, P, O, M> = Meta<Triple<Meta<S, M>, Meta<P, M>, Meta<O, M>>, M>;
/// Quad with heterogeneous component types, each wrapped in metadata `M`,
/// the whole quad itself wrapped in metadata.
pub type MetaQuadFull<S, P, O, G, M> = Meta<Quad<Meta<S, M>, Meta<P, M>, Meta<O, M>, Meta<G, M>>, M>;

impl<S, P, O> Triple<S, P, O>
where
    S: IsSubject + Strip,
    P: IsPredicate,
    O: IsObject + Strip,
    S::Stripped: IsSubject,
    O::Stripped: IsObject,
{
    /// Strips metadata from subject and object, leaving the predicate untouched.
    pub fn strip_all_but_predicate(self) -> Triple<S::Stripped, P, O::Stripped> {
        Triple(self.0.strip(), self.1, self.2.strip())
    }
}

impl<S, P, O, G> Quad<S, P, O, G>
where
    S: IsSubject + Strip,
    P: IsPredicate,
    O: IsObject + Strip,
    G: IsGraph + Strip,
    S::Stripped: IsSubject,
    O::Stripped: IsObject,
    G::Stripped: IsGraph,
{
    /// Strips metadata from subject, object, and graph, leaving the predicate untouched.
    pub fn strip_all_but_predicate(self) -> Quad<S::Stripped, P, O::Stripped, G::Stripped> {
        Quad(self.0.strip(), self.1, self.2.strip(), self.3.map(Strip::strip))
    }
}
