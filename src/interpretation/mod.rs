//! Resource interpretations.
use std::borrow::Cow;

use crate::{BlankId, CowId, CowLiteral, CowLocalTerm, CowTerm, IdRef, LiteralRef, LocalTermRef, TermRef, TripleTerm};

/// Borrow-or-own view of a triple term's three components, used by
/// [`ReverseInterpretation::triple_term_components_view`].
pub type TripleTermView<'a, R> = (Cow<'a, R>, Cow<'a, R>, Cow<'a, R>);

mod r#impl;
pub use r#impl::*;

pub mod fallible;
pub use fallible::FallibleInterpretation;

mod iri;
pub use iri::*;

mod blank_id;
pub use blank_id::*;

mod literal;
pub use literal::*;

mod id;
pub use id::*;

mod term;
pub use term::*;

use iri_rs::{Iri, IriBuf};

/// RDF resource interpretation.
pub trait Interpretation {
    type Resource;

    fn iri(&self, iri: Iri<&str>) -> Option<Self::Resource>;

    fn literal<'a>(&self, literal: impl Into<LiteralRef<'a>>) -> Option<Self::Resource>;

    fn term<'a>(&self, term: impl Into<TermRef<'a>>) -> Option<Self::Resource> {
        match term.into() {
            TermRef::Iri(iri) => self.iri(iri),
            TermRef::Literal(l) => self.literal(l),
        }
    }
}

impl<I: Interpretation> Interpretation for &I {
    type Resource = I::Resource;

    fn iri(&self, iri: Iri<&str>) -> Option<Self::Resource> {
        I::iri(*self, iri)
    }

    fn literal<'a>(&self, literal: impl Into<LiteralRef<'a>>) -> Option<Self::Resource> {
        I::literal(*self, literal)
    }
}

impl<I: Interpretation> Interpretation for &mut I {
    type Resource = I::Resource;

    fn iri(&self, iri: Iri<&str>) -> Option<Self::Resource> {
        I::iri(*self, iri)
    }

    fn literal<'a>(&self, literal: impl Into<LiteralRef<'a>>) -> Option<Self::Resource> {
        I::literal(*self, literal)
    }
}

/// Interpretation that can return an iterator over the known RDF resources.
pub trait TraversableInterpretation: Interpretation {
    type Resources<'a>: Iterator<Item = &'a Self::Resource>
    where
        Self: 'a;

    fn resources(&self) -> Self::Resources<'_>;
}

/// Interpretation that can spawn fresh new resources.
pub trait GenerativeInterpretation: Interpretation {
    /// Create a new resource.
    fn new_resource(&mut self) -> Self::Resource;
}

/// Interpretation that can spawn fresh new resources.
pub trait ConstGenerativeInterpretation: Interpretation {
    /// Create a new resource.
    fn new_resource(&self) -> Self::Resource;
}

impl<I: ConstGenerativeInterpretation> GenerativeInterpretation for I {
    fn new_resource(&mut self) -> Self::Resource {
        ConstGenerativeInterpretation::new_resource(self)
    }
}

/// Mutable interpretation.
pub trait InterpretationMut: Interpretation {
    fn insert_iri<'a>(&mut self, iri: impl Into<Cow<'a, IriBuf>>) -> Self::Resource;

    fn insert_literal<'a>(&mut self, literal: impl Into<CowLiteral<'a>>) -> Self::Resource;

    fn insert_term<'a>(&mut self, term: impl Into<CowTerm<'a>>) -> Self::Resource {
        match term.into() {
            CowTerm::Iri(iri) => self.insert_iri(iri),
            CowTerm::Literal(literal) => self.insert_literal(literal),
        }
    }
}

/// Reverse interpretation.
pub trait ReverseInterpretation: Interpretation {
    type Iris<'a>: Iterator<Item = Cow<'a, IriBuf>>
    where
        Self: 'a;
    type Literals<'a>: Iterator<Item = CowLiteral<'a>>
    where
        Self: 'a;

    fn iris_of<'a>(&'a self, resource: &'a Self::Resource) -> Self::Iris<'a>;

    fn literals_of<'a>(&'a self, resource: &'a Self::Resource) -> Self::Literals<'a>;

    fn terms_of<'a>(&'a self, resource: &'a Self::Resource) -> TermsOf<'a, Self> {
        TermsOf {
            iris: self.iris_of(resource),
            literals: self.literals_of(resource),
        }
    }

    /// RDF 1.2: returns the components of a triple-term resource as
    /// `(subject, predicate, object)` if `resource` denotes a triple term.
    /// Default returns `None` (no triple-term recognition); interpretations
    /// that store triple terms should override.
    ///
    /// Components are returned by value because the underlying triple-term
    /// body may use a different concrete type from `Self::Resource`
    /// (e.g. lexical bodies use `Id` for the subject, `IriBuf` for the
    /// predicate, both wrapped into `Self::Resource` for uniformity).
    ///
    /// For zero-clone descent during isomorphism — the dominant hot path —
    /// override [`Self::triple_term_components_view`] instead (or in addition).
    fn triple_term_components(&self, _resource: &Self::Resource) -> Option<(Self::Resource, Self::Resource, Self::Resource)> {
        None
    }

    /// Borrow-or-own view of triple-term components — the zero-clone
    /// counterpart of [`Self::triple_term_components`].
    ///
    /// Implementations whose component types are storage-natural to
    /// `Self::Resource` (e.g. an interpretation that stores triple terms
    /// using `LocalTerm`-typed components) should override and return
    /// [`Cow::Borrowed`] for those components. The default delegates to
    /// [`Self::triple_term_components`] and wraps each component in [`Cow::Owned`].
    ///
    /// Used internally by [`graph_equivalent_with`](crate::dataset::isomorphism::graph_equivalent_with)
    /// / [`dataset_equivalent_with`](crate::dataset::isomorphism::dataset_equivalent_with)
    /// during triple-term descent — overriding here removes per-descent
    /// clones for triple-term-heavy graphs.
    fn triple_term_components_view<'a>(&'a self, resource: &'a Self::Resource) -> Option<TripleTermView<'a, Self::Resource>>
    where
        Self::Resource: Clone,
    {
        self.triple_term_components(resource).map(|(s, p, o)| (Cow::Owned(s), Cow::Owned(p), Cow::Owned(o)))
    }

    fn is_blank_id(&self, resource: &Self::Resource) -> bool {
        self.terms_of(resource).next().is_none() && self.triple_term_components(resource).is_none()
    }
}

pub struct TermsOf<'a, I: 'a + ?Sized + ReverseInterpretation> {
    iris: I::Iris<'a>,
    literals: I::Literals<'a>,
}

impl<'a, I: 'a + ?Sized + ReverseInterpretation> Iterator for TermsOf<'a, I> {
    type Item = CowTerm<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iris.next().map(CowTerm::Iri).or_else(|| self.literals.next().map(CowTerm::Literal))
    }
}

pub trait LocalInterpretation: Interpretation {
    fn blank_id(&self, blank_id: &BlankId) -> Option<Self::Resource>;

    fn local_term<'a>(&self, term: impl Into<LocalTermRef<'a>>) -> Option<Self::Resource> {
        match term.into() {
            LocalTermRef::BlankId(blank_id) => self.blank_id(blank_id),
            LocalTermRef::Named(term) => self.term(term),
            LocalTermRef::Triple(t) => self.triple_term(t.clone()),
        }
    }

    /// Looks up a triple-term resource (RDF 1.2). Default implementation
    /// returns `None` — interpretations that store triple terms should
    /// override.
    fn triple_term(&self, _t: TripleTerm) -> Option<Self::Resource> {
        None
    }

    fn id<'a>(&self, id: impl Into<IdRef<'a>>) -> Option<Self::Resource> {
        match id.into() {
            IdRef::BlankId(blank_id) => self.blank_id(blank_id),
            IdRef::Iri(iri) => self.iri(iri),
        }
    }
}

pub trait LocalInterpretationMut: InterpretationMut {
    fn insert_blank_id<'a>(&mut self, blank_id: impl Into<Cow<'a, BlankId>>) -> Self::Resource;

    fn insert_local_term<'a>(&mut self, term: impl Into<CowLocalTerm<'a>>) -> Self::Resource {
        match term.into() {
            CowLocalTerm::BlankId(blank_id) => self.insert_blank_id(blank_id),
            CowLocalTerm::Named(term) => self.insert_term(term),
            CowLocalTerm::Triple(t) => self.insert_triple_term(t.into_owned()),
        }
    }

    /// Inserts a triple-term resource (RDF 1.2). Default implementation
    /// panics — interpretations that need to ingest triple-term-bearing
    /// graphs must override this method.
    fn insert_triple_term(&mut self, _t: TripleTerm) -> Self::Resource {
        unimplemented!("triple-term insertion not implemented by this LocalInterpretationMut")
    }

    fn insert_id<'a>(&mut self, term: impl Into<CowId<'a>>) -> Self::Resource {
        match term.into() {
            CowId::BlankId(blank_id) => self.insert_blank_id(blank_id),
            CowId::Iri(iri) => self.insert_iri(iri),
        }
    }
}

pub trait ReverseLocalInterpretation: ReverseInterpretation {
    type BlankIds<'a>: Iterator<Item = Cow<'a, BlankId>>
    where
        Self: 'a;

    fn blank_ids_of<'a>(&'a self, resource: &'a Self::Resource) -> Self::BlankIds<'a>;

    fn local_terms_of<'a>(&'a self, resource: &'a Self::Resource) -> LocalTermsOf<'a, Self> {
        LocalTermsOf {
            terms: self.terms_of(resource),
            blank_ids: self.blank_ids_of(resource),
        }
    }
}

pub struct LocalTermsOf<'a, I: 'a + ?Sized + ReverseLocalInterpretation> {
    terms: TermsOf<'a, I>,
    blank_ids: I::BlankIds<'a>,
}

impl<'a, I: 'a + ?Sized + ReverseLocalInterpretation> Iterator for LocalTermsOf<'a, I> {
    type Item = CowLocalTerm<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.terms
            .next()
            .map(CowLocalTerm::Named)
            .or_else(|| self.blank_ids.next().map(CowLocalTerm::BlankId))
    }
}
