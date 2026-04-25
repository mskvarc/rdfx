//! Metadata-tagged terms and triples/quads, gated on the `meta` feature.
//!
//! Provides a generic [`Meta<T, M>`] wrapper that pairs any value with metadata
//! (e.g. a [`locspan::Span`] or [`locspan::Location`]). The [`Strip`] trait
//! discards the metadata layer.
//!
//! # Examples
//!
//! Round-tripping a tagged term through [`Strip`]:
//!
//! ```
//! use rdf_rs::{BlankIdBuf, LocalTerm, meta::{Meta, Span, Strip}};
//!
//! let term = LocalTerm::BlankId(BlankIdBuf::new("_:b0".to_owned()).unwrap());
//! let tagged = Meta(term.clone(), Span::new(0, 5));
//! assert_eq!(tagged.strip(), term);
//! ```
//!
//! Using [`Meta`] as a dataset key to keep otherwise-equal quads distinct
//! by their metadata:
//!
//! ```
//! use rdf_rs::{BlankIdBuf, LocalTerm, Quad, dataset::BTreeDataset, meta::{Meta, Span}};
//!
//! let s = LocalTerm::BlankId(BlankIdBuf::new("_:s".to_owned()).unwrap());
//! let p = LocalTerm::iri(iri_rs::IriBuf::new("http://example.org/p".to_owned()).unwrap());
//! let o = LocalTerm::iri(iri_rs::IriBuf::new("http://example.org/o".to_owned()).unwrap());
//!
//! let mut d: BTreeDataset<Meta<LocalTerm, Span>> = BTreeDataset::new();
//! d.insert(Quad(Meta(s.clone(), Span::new(0, 1)), Meta(p.clone(), Span::new(0, 1)), Meta(o.clone(), Span::new(0, 1)), None));
//! d.insert(Quad(Meta(s, Span::new(2, 3)), Meta(p, Span::new(2, 3)), Meta(o, Span::new(2, 3)), None));
//! assert_eq!(d.len(), 2);
//! ```

use crate::{BlankIdBuf, Id, Literal, LiteralType, LocalTerm, Quad, Term, Triple};
use iri_rs::IriBuf;

#[doc(no_inline)]
pub use locspan::{Location, Span};

/// A value of type `T` annotated with metadata `M`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Meta<T, M>(pub T, pub M);

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

identity_strip!(BlankIdBuf, IriBuf, Id, Literal, LiteralType, Term, LocalTerm);

impl<S: Strip, P: Strip, O: Strip> Strip for Triple<S, P, O> {
    type Stripped = Triple<S::Stripped, P::Stripped, O::Stripped>;

    fn strip(self) -> Self::Stripped {
        Triple(self.0.strip(), self.1.strip(), self.2.strip())
    }
}

impl<S: Strip, P: Strip, O: Strip, G: Strip> Strip for Quad<S, P, O, G> {
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

impl<S: Strip, P, O: Strip> Triple<S, P, O> {
    /// Strips metadata from subject and object, leaving the predicate untouched.
    pub fn strip_all_but_predicate(self) -> Triple<S::Stripped, P, O::Stripped> {
        Triple(self.0.strip(), self.1, self.2.strip())
    }
}

impl<S: Strip, P, O: Strip, G: Strip> Quad<S, P, O, G> {
    /// Strips metadata from subject, object, and graph, leaving the predicate untouched.
    pub fn strip_all_but_predicate(self) -> Quad<S::Stripped, P, O::Stripped, G::Stripped> {
        Quad(self.0.strip(), self.1, self.2.strip(), self.3.map(Strip::strip))
    }
}
