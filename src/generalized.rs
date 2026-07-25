//! Generalized RDF (RDF 1.1 §A / RDF 1.2 §A.1) types — no position invariants.
//!
//! Per [RDF 1.1 Concepts §A][s] / [RDF 1.2 Concepts §A.1][s2], generalized RDF
//! relaxes the abstract syntax:
//! - subjects may be literals,
//! - predicates may be blank nodes or literals,
//! - graph names may be literals,
//! - triple terms may appear in any position (RDF 1.2 generalization).
//!
//! Use [`GeneralizedTriple`] / [`GeneralizedQuad`] when interoperating with
//! generalized syntaxes; use [`Triple`] / [`Quad`] for strict RDF 1.1 / 1.2.
//!
//! Two flavors of generalized "local term" exist:
//! - [`LocalTerm`] (default) — strict-RDF triple-term bodies inside any
//!   triple-term object. Sufficient for most generalized-Turtle inputs.
//! - [`GeneralizedLocalTerm`] — fully generalized: triple-term bodies may
//!   themselves carry literal subjects / blank predicates / nested triple
//!   terms. Reachable via [`FullyGeneralizedTriple`] / [`FullyGeneralizedQuad`].
//!
//! [s]:  https://www.w3.org/TR/rdf11-concepts/#section-generalized-rdf
//! [s2]: https://www.w3.org/TR/rdf12-concepts/#section-generalized-rdf

use std::{cmp::Ordering, fmt};

use iri_rs::{Iri, IriBuf};

use crate::{BlankId, BlankIdBuf, Id, IsGraph, IsObject, IsPredicate, IsSubject, Literal, LiteralRef, LocalTerm, Quad, RdfDisplay, Term, Triple, __seal};

/// Reasons a [`GeneralizedTriple`] or [`GeneralizedQuad`] cannot be converted
/// to a strict [`Triple`] / [`Quad`].
#[derive(Debug, thiserror::Error)]
pub enum GeneralizationError {
    #[error("literal in subject position")]
    /// A literal sits in subject position.
    LiteralSubject,
    #[error("literal in predicate position")]
    /// A literal sits in predicate position.
    LiteralPredicate,
    #[error("blank node in predicate position")]
    /// A blank node sits in predicate position.
    BlankPredicate,
    #[error("literal in graph position")]
    /// A literal sits in graph position.
    LiteralGraph,
    /// A triple term sits in subject position. Strict RDF 1.2 only allows
    /// triple terms in object position.
    #[error("triple term in subject position")]
    TripleTermSubject,
    /// A triple term sits in predicate position.
    #[error("triple term in predicate position")]
    TripleTermPredicate,
    /// A triple term sits in graph-name position.
    #[error("triple term in graph position")]
    TripleTermGraph,
}

/// Generalized RDF triple — no position bounds on its components.
#[derive(Clone, Copy, Eq, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeneralizedTriple<S = LocalTerm, P = S, O = S>(pub S, pub P, pub O);

impl<S, P, O> GeneralizedTriple<S, P, O> {
    /// Creates a generalized triple from its components.
    pub const fn new(subject: S, predicate: P, object: O) -> Self {
        Self(subject, predicate, object)
    }

    /// Returns the subject of this triple.
    pub const fn subject(&self) -> &S {
        &self.0
    }

    /// Returns the predicate of this triple.
    pub const fn predicate(&self) -> &P {
        &self.1
    }

    /// Returns the object of this triple.
    pub const fn object(&self) -> &O {
        &self.2
    }

    /// Consumes this triple, returning its subject.
    pub fn into_subject(self) -> S {
        self.0
    }

    /// Consumes this triple, returning its predicate.
    pub fn into_predicate(self) -> P {
        self.1
    }

    /// Consumes this triple, returning its object.
    pub fn into_object(self) -> O {
        self.2
    }

    /// Consumes this triple, returning each component separately.
    pub fn into_parts(self) -> (S, P, O) {
        (self.0, self.1, self.2)
    }

    /// Lifts this triple into a generalized quad in the given graph.
    pub fn into_quad<G>(self, graph: Option<G>) -> GeneralizedQuad<S, P, O, G> {
        GeneralizedQuad(self.0, self.1, self.2, graph)
    }

    /// Maps the subject of this triple with `f`.
    pub fn map_subject<U>(self, f: impl FnOnce(S) -> U) -> GeneralizedTriple<U, P, O> {
        GeneralizedTriple(f(self.0), self.1, self.2)
    }

    /// Maps the predicate of this triple with `f`.
    pub fn map_predicate<U>(self, f: impl FnOnce(P) -> U) -> GeneralizedTriple<S, U, O> {
        GeneralizedTriple(self.0, f(self.1), self.2)
    }

    /// Maps the object of this triple with `f`.
    pub fn map_object<U>(self, f: impl FnOnce(O) -> U) -> GeneralizedTriple<S, P, U> {
        GeneralizedTriple(self.0, self.1, f(self.2))
    }

    /// Borrows the components of this triple.
    pub const fn as_ref(&self) -> GeneralizedTriple<&S, &P, &O> {
        GeneralizedTriple(&self.0, &self.1, &self.2)
    }
}

impl<S, P, O> GeneralizedTriple<&S, &P, &O> {
    /// Clones the borrowed components of this triple.
    pub fn cloned(&self) -> GeneralizedTriple<S, P, O>
    where
        S: Clone,
        P: Clone,
        O: Clone,
    {
        GeneralizedTriple(self.0.clone(), self.1.clone(), self.2.clone())
    }

    /// Consumes this triple, cloning its borrowed components.
    pub fn into_cloned(self) -> GeneralizedTriple<S, P, O>
    where
        S: Clone,
        P: Clone,
        O: Clone,
    {
        GeneralizedTriple(self.0.clone(), self.1.clone(), self.2.clone())
    }

    /// Copies the borrowed components of this triple.
    pub const fn copied(&self) -> GeneralizedTriple<S, P, O>
    where
        S: Copy,
        P: Copy,
        O: Copy,
    {
        GeneralizedTriple(*self.0, *self.1, *self.2)
    }

    /// Consumes this triple, copying its borrowed components.
    pub const fn into_copied(self) -> GeneralizedTriple<S, P, O>
    where
        S: Copy,
        P: Copy,
        O: Copy,
    {
        GeneralizedTriple(*self.0, *self.1, *self.2)
    }
}

impl<T> GeneralizedTriple<T, T, T> {
    /// Maps every component of this triple with `f`.
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> GeneralizedTriple<U, U, U> {
        GeneralizedTriple(f(self.0), f(self.1), f(self.2))
    }
}

impl GeneralizedTriple<LocalTerm, LocalTerm, LocalTerm> {
    /// Tries to narrow this generalized triple into a strict [`Triple`].
    /// Fails if a literal sits in subject position, the predicate is a
    /// blank node, literal, or triple term, or a triple term sits in subject
    /// position.
    pub fn try_into_triple(self) -> Result<Triple<Id, IriBuf, LocalTerm>, GeneralizationError> {
        let GeneralizedTriple(s, p, o) = self;
        let s = local_term_to_id(s)?;
        let p = local_term_to_iri(p)?;
        Ok(Triple(s, p, o))
    }
}

impl<S: IsSubject, P: IsPredicate, O: IsObject> From<Triple<S, P, O>> for GeneralizedTriple<S, P, O> {
    fn from(t: Triple<S, P, O>) -> Self {
        GeneralizedTriple(t.0, t.1, t.2)
    }
}

impl<S1, P1, O1, S2, P2, O2> PartialEq<GeneralizedTriple<S2, P2, O2>> for GeneralizedTriple<S1, P1, O1>
where
    S1: PartialEq<S2>,
    P1: PartialEq<P2>,
    O1: PartialEq<O2>,
{
    fn eq(&self, other: &GeneralizedTriple<S2, P2, O2>) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl<S1, P1, O1, S2, P2, O2> PartialOrd<GeneralizedTriple<S2, P2, O2>> for GeneralizedTriple<S1, P1, O1>
where
    S1: PartialOrd<S2>,
    P1: PartialOrd<P2>,
    O1: PartialOrd<O2>,
{
    fn partial_cmp(&self, other: &GeneralizedTriple<S2, P2, O2>) -> Option<Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(Ordering::Equal) => match self.1.partial_cmp(&other.1) {
                Some(Ordering::Equal) => self.2.partial_cmp(&other.2),
                cmp => cmp,
            },
            cmp => cmp,
        }
    }
}

impl<S: RdfDisplay, P: RdfDisplay, O: RdfDisplay> fmt::Display for GeneralizedTriple<S, P, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.0.rdf_display(), self.1.rdf_display(), self.2.rdf_display())
    }
}

impl<S: RdfDisplay, P: RdfDisplay, O: RdfDisplay> RdfDisplay for GeneralizedTriple<S, P, O> {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.0.rdf_display(), self.1.rdf_display(), self.2.rdf_display())
    }
}

/// Generalized RDF quad — no position bounds on its components.
#[derive(Clone, Copy, Eq, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeneralizedQuad<S = LocalTerm, P = S, O = S, G = S>(pub S, pub P, pub O, pub Option<G>);

impl<S, P, O, G> GeneralizedQuad<S, P, O, G> {
    /// Creates a generalized quad from its components.
    pub const fn new(subject: S, predicate: P, object: O, graph: Option<G>) -> Self {
        Self(subject, predicate, object, graph)
    }

    /// Returns the subject of this quad.
    pub const fn subject(&self) -> &S {
        &self.0
    }

    /// Returns the predicate of this quad.
    pub const fn predicate(&self) -> &P {
        &self.1
    }

    /// Returns the object of this quad.
    pub const fn object(&self) -> &O {
        &self.2
    }

    /// Returns the graph of this quad.
    pub const fn graph(&self) -> Option<&G> {
        self.3.as_ref()
    }

    /// Consumes this quad, returning its subject.
    pub fn into_subject(self) -> S {
        self.0
    }

    /// Consumes this quad, returning its predicate.
    pub fn into_predicate(self) -> P {
        self.1
    }

    /// Consumes this quad, returning its object.
    pub fn into_object(self) -> O {
        self.2
    }

    /// Consumes this quad, returning its graph.
    pub fn into_graph(self) -> Option<G> {
        self.3
    }

    /// Consumes this quad, returning each component separately.
    pub fn into_parts(self) -> (S, P, O, Option<G>) {
        (self.0, self.1, self.2, self.3)
    }

    /// Splits this quad into a generalized triple and its graph.
    pub fn into_triple(self) -> (GeneralizedTriple<S, P, O>, Option<G>) {
        (GeneralizedTriple(self.0, self.1, self.2), self.3)
    }

    /// Maps the subject of this quad with `f`.
    pub fn map_subject<U>(self, f: impl FnOnce(S) -> U) -> GeneralizedQuad<U, P, O, G> {
        GeneralizedQuad(f(self.0), self.1, self.2, self.3)
    }

    /// Maps the predicate of this quad with `f`.
    pub fn map_predicate<U>(self, f: impl FnOnce(P) -> U) -> GeneralizedQuad<S, U, O, G> {
        GeneralizedQuad(self.0, f(self.1), self.2, self.3)
    }

    /// Maps the object of this quad with `f`.
    pub fn map_object<U>(self, f: impl FnOnce(O) -> U) -> GeneralizedQuad<S, P, U, G> {
        GeneralizedQuad(self.0, self.1, f(self.2), self.3)
    }

    /// Maps the graph of this quad with `f`.
    pub fn map_graph<U>(self, f: impl FnOnce(Option<G>) -> Option<U>) -> GeneralizedQuad<S, P, O, U> {
        GeneralizedQuad(self.0, self.1, self.2, f(self.3))
    }

    /// Returns this quad moved into the given graph.
    pub fn with_graph(self, g: Option<G>) -> Self {
        Self(self.0, self.1, self.2, g)
    }

    /// Maps every component of this quad with its own function.
    pub fn map_all<S2, P2, O2, G2>(
        self,
        s: impl FnOnce(S) -> S2,
        p: impl FnOnce(P) -> P2,
        o: impl FnOnce(O) -> O2,
        g: impl FnOnce(Option<G>) -> Option<G2>,
    ) -> GeneralizedQuad<S2, P2, O2, G2> {
        GeneralizedQuad(s(self.0), p(self.1), o(self.2), g(self.3))
    }

    /// Borrows the components of this quad.
    pub const fn as_ref(&self) -> GeneralizedQuad<&S, &P, &O, &G> {
        GeneralizedQuad(&self.0, &self.1, &self.2, self.3.as_ref())
    }
}

impl<S, P, O, G> GeneralizedQuad<&S, &P, &O, &G> {
    /// Clones the borrowed components of this quad.
    pub fn cloned(&self) -> GeneralizedQuad<S, P, O, G>
    where
        S: Clone,
        P: Clone,
        O: Clone,
        G: Clone,
    {
        GeneralizedQuad(self.0.clone(), self.1.clone(), self.2.clone(), self.3.cloned())
    }

    /// Consumes this quad, cloning its borrowed components.
    pub fn into_cloned(self) -> GeneralizedQuad<S, P, O, G>
    where
        S: Clone,
        P: Clone,
        O: Clone,
        G: Clone,
    {
        GeneralizedQuad(self.0.clone(), self.1.clone(), self.2.clone(), self.3.cloned())
    }

    /// Copies the borrowed components of this quad.
    pub const fn copied(&self) -> GeneralizedQuad<S, P, O, G>
    where
        S: Copy,
        P: Copy,
        O: Copy,
        G: Copy,
    {
        GeneralizedQuad(*self.0, *self.1, *self.2, self.3.copied())
    }

    /// Consumes this quad, copying its borrowed components.
    pub const fn into_copied(self) -> GeneralizedQuad<S, P, O, G>
    where
        S: Copy,
        P: Copy,
        O: Copy,
        G: Copy,
    {
        GeneralizedQuad(*self.0, *self.1, *self.2, self.3.copied())
    }
}

impl<T> GeneralizedQuad<T, T, T, T> {
    /// Maps every component of this quad with `f`.
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> GeneralizedQuad<U, U, U, U> {
        GeneralizedQuad(f(self.0), f(self.1), f(self.2), self.3.map(f))
    }
}

impl GeneralizedQuad<LocalTerm, LocalTerm, LocalTerm, LocalTerm> {
    /// Tries to narrow this generalized quad into a strict [`Quad`].
    pub fn try_into_quad(self) -> Result<Quad<Id, IriBuf, LocalTerm, Id>, GeneralizationError> {
        let GeneralizedQuad(s, p, o, g) = self;
        let s = local_term_to_id(s)?;
        let p = local_term_to_iri(p)?;
        let g = match g {
            Some(t) => Some(local_term_to_graph_id(t)?),
            None => None,
        };
        Ok(Quad(s, p, o, g))
    }
}

fn local_term_to_id(t: LocalTerm) -> Result<Id, GeneralizationError> {
    match t {
        LocalTerm::BlankId(b) => Ok(Id::BlankId(b)),
        LocalTerm::Named(Term::Iri(iri)) => Ok(Id::Iri(iri)),
        LocalTerm::Named(Term::Literal(_)) => Err(GeneralizationError::LiteralSubject),
        LocalTerm::Triple(_) => Err(GeneralizationError::TripleTermSubject),
    }
}

fn local_term_to_iri(t: LocalTerm) -> Result<IriBuf, GeneralizationError> {
    match t {
        LocalTerm::BlankId(_) => Err(GeneralizationError::BlankPredicate),
        LocalTerm::Named(Term::Iri(iri)) => Ok(iri),
        LocalTerm::Named(Term::Literal(_)) => Err(GeneralizationError::LiteralPredicate),
        LocalTerm::Triple(_) => Err(GeneralizationError::TripleTermPredicate),
    }
}

fn local_term_to_graph_id(t: LocalTerm) -> Result<Id, GeneralizationError> {
    match t {
        LocalTerm::BlankId(b) => Ok(Id::BlankId(b)),
        LocalTerm::Named(Term::Iri(iri)) => Ok(Id::Iri(iri)),
        LocalTerm::Named(Term::Literal(_)) => Err(GeneralizationError::LiteralGraph),
        LocalTerm::Triple(_) => Err(GeneralizationError::TripleTermGraph),
    }
}

impl<S: IsSubject, P: IsPredicate, O: IsObject, G: IsGraph> From<Quad<S, P, O, G>> for GeneralizedQuad<S, P, O, G> {
    fn from(q: Quad<S, P, O, G>) -> Self {
        GeneralizedQuad(q.0, q.1, q.2, q.3)
    }
}

impl<S1, P1, O1, G1, S2, P2, O2, G2> PartialEq<GeneralizedQuad<S2, P2, O2, G2>> for GeneralizedQuad<S1, P1, O1, G1>
where
    S1: PartialEq<S2>,
    P1: PartialEq<P2>,
    O1: PartialEq<O2>,
    G1: PartialEq<G2>,
{
    fn eq(&self, other: &GeneralizedQuad<S2, P2, O2, G2>) -> bool {
        self.0 == other.0
            && self.1 == other.1
            && self.2 == other.2
            && match (&self.3, &other.3) {
                (Some(a), Some(b)) => a == b,
                (None, None) => true,
                _ => false,
            }
    }
}

impl<S1, P1, O1, G1, S2, P2, O2, G2> PartialOrd<GeneralizedQuad<S2, P2, O2, G2>> for GeneralizedQuad<S1, P1, O1, G1>
where
    S1: PartialOrd<S2>,
    P1: PartialOrd<P2>,
    O1: PartialOrd<O2>,
    G1: PartialOrd<G2>,
{
    fn partial_cmp(&self, other: &GeneralizedQuad<S2, P2, O2, G2>) -> Option<Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(Ordering::Equal) => match self.1.partial_cmp(&other.1) {
                Some(Ordering::Equal) => match self.2.partial_cmp(&other.2) {
                    Some(Ordering::Equal) => match (&self.3, &other.3) {
                        (Some(a), Some(b)) => a.partial_cmp(b),
                        (Some(_), None) => Some(Ordering::Greater),
                        (None, Some(_)) => Some(Ordering::Less),
                        (None, None) => Some(Ordering::Equal),
                    },
                    cmp => cmp,
                },
                cmp => cmp,
            },
            cmp => cmp,
        }
    }
}

impl<S: RdfDisplay, P: RdfDisplay, O: RdfDisplay, G: RdfDisplay> fmt::Display for GeneralizedQuad<S, P, O, G> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.graph() {
            Some(graph) => write!(
                f,
                "{} {} {} {}",
                self.0.rdf_display(),
                self.1.rdf_display(),
                self.2.rdf_display(),
                graph.rdf_display()
            ),
            None => write!(f, "{} {} {}", self.0.rdf_display(), self.1.rdf_display(), self.2.rdf_display()),
        }
    }
}

impl<S: RdfDisplay, P: RdfDisplay, O: RdfDisplay, G: RdfDisplay> RdfDisplay for GeneralizedQuad<S, P, O, G> {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.graph() {
            Some(graph) => write!(
                f,
                "{} {} {} {}",
                self.0.rdf_display(),
                self.1.rdf_display(),
                self.2.rdf_display(),
                graph.rdf_display()
            ),
            None => write!(f, "{} {} {}", self.0.rdf_display(), self.1.rdf_display(), self.2.rdf_display()),
        }
    }
}

// =========================================================================
// Fully-generalized local term (RDF 1.2 §A.1)
// =========================================================================

/// Fully-generalized local term — like [`LocalTerm`] but its triple-term
/// bodies may themselves carry literal subjects, blank predicates, and
/// nested triple terms in any position.
///
/// Reachable as the component type of [`FullyGeneralizedTriple`] /
/// [`FullyGeneralizedQuad`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeneralizedLocalTerm {
    /// A blank node identifier.
    BlankId(BlankIdBuf),
    /// An IRI or a literal.
    Named(Term),
    /// Fully-generalized triple term — body components are themselves
    /// [`GeneralizedLocalTerm`].
    Triple(Box<GeneralizedTriple<GeneralizedLocalTerm>>),
}

impl GeneralizedLocalTerm {
    /// Creates a term from an IRI.
    pub const fn iri(iri: IriBuf) -> Self {
        Self::Named(Term::Iri(iri))
    }

    /// Creates a term from a literal.
    pub const fn literal(literal: Literal) -> Self {
        Self::Named(Term::Literal(literal))
    }

    /// Creates a term from a triple term.
    pub fn triple(t: GeneralizedTriple<GeneralizedLocalTerm>) -> Self {
        Self::Triple(Box::new(t))  // GeneralizedLocalTerm keeps Box (no shared-clone story; users construct in tree-shape)
    }

    /// Checks whether this term is a blank node identifier.
    pub const fn is_blank_id(&self) -> bool {
        matches!(self, Self::BlankId(_))
    }

    /// Checks whether this term is a triple term.
    pub const fn is_triple(&self) -> bool {
        matches!(self, Self::Triple(_))
    }

    /// Returns the blank node identifier of this term, if it is one.
    pub fn as_blank_id(&self) -> Option<&BlankId> {
        match self {
            Self::BlankId(b) => Some(b),
            _ => None,
        }
    }

    /// Returns the IRI of this term, if it is one.
    pub fn as_iri(&self) -> Option<Iri<&str>> {
        match self {
            Self::Named(Term::Iri(iri)) => Some(iri.as_ref()),
            _ => None,
        }
    }

    /// Returns the literal of this term, if it is one.
    pub fn as_literal(&self) -> Option<LiteralRef<'_>> {
        match self {
            Self::Named(Term::Literal(l)) => Some(l.as_ref()),
            _ => None,
        }
    }

    /// Returns the triple term of this term, if it is one.
    pub fn as_triple(&self) -> Option<&GeneralizedTriple<GeneralizedLocalTerm>> {
        match self {
            Self::Triple(t) => Some(t),
            _ => None,
        }
    }

    /// Narrows this fully-generalized term to a strict [`LocalTerm`].
    pub fn try_into_local_term(self) -> Result<LocalTerm, GeneralizationError> {
        match self {
            Self::BlankId(b) => Ok(LocalTerm::BlankId(b)),
            Self::Named(t) => Ok(LocalTerm::Named(t)),
            Self::Triple(b) => {
                let GeneralizedTriple(s, p, o) = *b;
                let s = generalized_to_id(s, GeneralizationError::LiteralSubject)?;
                let p = generalized_to_iri(p)?;
                let o = o.try_into_local_term()?;
                Ok(LocalTerm::triple(Triple(s, p, o)))
            }
        }
    }
}

impl From<LocalTerm> for GeneralizedLocalTerm {
    fn from(value: LocalTerm) -> Self {
        match value {
            LocalTerm::BlankId(b) => Self::BlankId(b),
            LocalTerm::Named(t) => Self::Named(t),
            LocalTerm::Triple(b) => {
                let Triple(s, p, o) = std::sync::Arc::try_unwrap(b).unwrap_or_else(|arc| (*arc).clone());
                let s = match s {
                    Id::Iri(iri) => Self::Named(Term::Iri(iri)),
                    Id::BlankId(b) => Self::BlankId(b),
                };
                let p = Self::Named(Term::Iri(p));
                let o = Self::from(o);
                Self::triple(GeneralizedTriple(s, p, o))
            }
        }
    }
}

impl From<BlankIdBuf> for GeneralizedLocalTerm {
    fn from(value: BlankIdBuf) -> Self {
        Self::BlankId(value)
    }
}

impl From<IriBuf> for GeneralizedLocalTerm {
    fn from(value: IriBuf) -> Self {
        Self::iri(value)
    }
}

impl From<Literal> for GeneralizedLocalTerm {
    fn from(value: Literal) -> Self {
        Self::literal(value)
    }
}

impl From<Term> for GeneralizedLocalTerm {
    fn from(value: Term) -> Self {
        Self::Named(value)
    }
}

impl fmt::Display for GeneralizedLocalTerm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(b) => b.fmt(f),
            Self::Named(t) => t.fmt(f),
            Self::Triple(t) => {
                f.write_str("<<( ")?;
                t.rdf_fmt(f)?;
                f.write_str(" )>>")
            }
        }
    }
}

impl RdfDisplay for GeneralizedLocalTerm {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(b) => b.rdf_fmt(f),
            Self::Named(t) => t.rdf_fmt(f),
            Self::Triple(t) => {
                f.write_str("<<( ")?;
                t.rdf_fmt(f)?;
                f.write_str(" )>>")
            }
        }
    }
}

// Bare position-trait impls — fully generalized terms are usable in every
// position. No `Resource` impl (consistent with how `LocalTerm` is sealed:
// only the positions strictly required by RDF, not as a free-form resource).
impl __seal::Sealed for GeneralizedLocalTerm {}
impl IsSubject for GeneralizedLocalTerm {}
impl IsPredicate for GeneralizedLocalTerm {}
impl IsObject for GeneralizedLocalTerm {}
impl IsGraph for GeneralizedLocalTerm {}

/// Fully-generalized triple — body components are [`GeneralizedLocalTerm`].
pub type FullyGeneralizedTriple = GeneralizedTriple<GeneralizedLocalTerm>;

/// Fully-generalized quad — body components are [`GeneralizedLocalTerm`].
pub type FullyGeneralizedQuad = GeneralizedQuad<GeneralizedLocalTerm>;

impl FullyGeneralizedTriple {
    /// Tries to narrow this fully-generalized triple into a strict [`Triple`].
    pub fn try_into_strict_triple(self) -> Result<Triple<Id, IriBuf, LocalTerm>, GeneralizationError> {
        let GeneralizedTriple(s, p, o) = self;
        let s = generalized_to_id(s, GeneralizationError::LiteralSubject)?;
        let p = generalized_to_iri(p)?;
        let o = o.try_into_local_term()?;
        Ok(Triple(s, p, o))
    }
}

impl FullyGeneralizedQuad {
    /// Tries to narrow this fully-generalized quad into a strict [`Quad`].
    pub fn try_into_strict_quad(self) -> Result<Quad<Id, IriBuf, LocalTerm, Id>, GeneralizationError> {
        let GeneralizedQuad(s, p, o, g) = self;
        let s = generalized_to_id(s, GeneralizationError::LiteralSubject)?;
        let p = generalized_to_iri(p)?;
        let o = o.try_into_local_term()?;
        let g = match g {
            Some(t) => Some(generalized_to_id(t, GeneralizationError::LiteralGraph)?),
            None => None,
        };
        Ok(Quad(s, p, o, g))
    }
}

fn generalized_to_id(t: GeneralizedLocalTerm, on_literal: GeneralizationError) -> Result<Id, GeneralizationError> {
    match t {
        GeneralizedLocalTerm::BlankId(b) => Ok(Id::BlankId(b)),
        GeneralizedLocalTerm::Named(Term::Iri(iri)) => Ok(Id::Iri(iri)),
        GeneralizedLocalTerm::Named(Term::Literal(_)) => Err(on_literal),
        GeneralizedLocalTerm::Triple(_) => Err(match on_literal {
            GeneralizationError::LiteralGraph => GeneralizationError::TripleTermGraph,
            _ => GeneralizationError::TripleTermSubject,
        }),
    }
}

fn generalized_to_iri(t: GeneralizedLocalTerm) -> Result<IriBuf, GeneralizationError> {
    match t {
        GeneralizedLocalTerm::BlankId(_) => Err(GeneralizationError::BlankPredicate),
        GeneralizedLocalTerm::Named(Term::Iri(iri)) => Ok(iri),
        GeneralizedLocalTerm::Named(Term::Literal(_)) => Err(GeneralizationError::LiteralPredicate),
        GeneralizedLocalTerm::Triple(_) => Err(GeneralizationError::TripleTermPredicate),
    }
}

