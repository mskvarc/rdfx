//! Generalized RDF (RDF 1.1 §A) types — no position invariants.
//!
//! Per [RDF 1.1 Concepts §A][s], generalized RDF relaxes the abstract syntax:
//! - subjects may be literals,
//! - predicates may be blank nodes or literals,
//! - graph names may be literals.
//!
//! Use [`GeneralizedTriple`] / [`GeneralizedQuad`] when interoperating with
//! generalized syntaxes; use [`Triple`] / [`Quad`] for strict RDF 1.1.
//!
//! [s]: https://www.w3.org/TR/rdf11-concepts/#section-generalized-rdf

use std::{cmp::Ordering, fmt};

use iri_rs::IriBuf;

use crate::{Id, IsGraph, IsObject, IsPredicate, IsSubject, LocalTerm, Quad, RdfDisplay, Term, Triple};

/// Reasons a [`GeneralizedTriple`] or [`GeneralizedQuad`] cannot be converted
/// to a strict [`Triple`] / [`Quad`].
#[derive(Debug, thiserror::Error)]
pub enum GeneralizationError {
    #[error("literal in subject position")]
    LiteralSubject,
    #[error("literal in predicate position")]
    LiteralPredicate,
    #[error("blank node in predicate position")]
    BlankPredicate,
    #[error("literal in graph position")]
    LiteralGraph,
}

/// Generalized RDF triple (RDF 1.1 §A) — no position bounds on its components.
#[derive(Clone, Copy, Eq, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeneralizedTriple<S = LocalTerm, P = S, O = S>(pub S, pub P, pub O);

impl<S, P, O> GeneralizedTriple<S, P, O> {
    pub const fn new(subject: S, predicate: P, object: O) -> Self {
        Self(subject, predicate, object)
    }

    pub const fn subject(&self) -> &S {
        &self.0
    }

    pub const fn predicate(&self) -> &P {
        &self.1
    }

    pub const fn object(&self) -> &O {
        &self.2
    }

    pub fn into_subject(self) -> S {
        self.0
    }

    pub fn into_predicate(self) -> P {
        self.1
    }

    pub fn into_object(self) -> O {
        self.2
    }

    pub fn into_parts(self) -> (S, P, O) {
        (self.0, self.1, self.2)
    }

    pub fn into_quad<G>(self, graph: Option<G>) -> GeneralizedQuad<S, P, O, G> {
        GeneralizedQuad(self.0, self.1, self.2, graph)
    }

    pub fn map_subject<U>(self, f: impl FnOnce(S) -> U) -> GeneralizedTriple<U, P, O> {
        GeneralizedTriple(f(self.0), self.1, self.2)
    }

    pub fn map_predicate<U>(self, f: impl FnOnce(P) -> U) -> GeneralizedTriple<S, U, O> {
        GeneralizedTriple(self.0, f(self.1), self.2)
    }

    pub fn map_object<U>(self, f: impl FnOnce(O) -> U) -> GeneralizedTriple<S, P, U> {
        GeneralizedTriple(self.0, self.1, f(self.2))
    }

    pub const fn as_ref(&self) -> GeneralizedTriple<&S, &P, &O> {
        GeneralizedTriple(&self.0, &self.1, &self.2)
    }
}

impl<S, P, O> GeneralizedTriple<&S, &P, &O> {
    pub fn cloned(&self) -> GeneralizedTriple<S, P, O>
    where
        S: Clone,
        P: Clone,
        O: Clone,
    {
        GeneralizedTriple(self.0.clone(), self.1.clone(), self.2.clone())
    }

    pub fn into_cloned(self) -> GeneralizedTriple<S, P, O>
    where
        S: Clone,
        P: Clone,
        O: Clone,
    {
        GeneralizedTriple(self.0.clone(), self.1.clone(), self.2.clone())
    }

    pub const fn copied(&self) -> GeneralizedTriple<S, P, O>
    where
        S: Copy,
        P: Copy,
        O: Copy,
    {
        GeneralizedTriple(*self.0, *self.1, *self.2)
    }

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
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> GeneralizedTriple<U, U, U> {
        GeneralizedTriple(f(self.0), f(self.1), f(self.2))
    }
}

impl GeneralizedTriple<LocalTerm, LocalTerm, LocalTerm> {
    /// Tries to narrow this generalized triple into a strict [`Triple`].
    /// Fails if a literal sits in subject position, or the predicate is a
    /// blank node or literal.
    pub fn try_into_triple(self) -> Result<Triple<Id, IriBuf, LocalTerm>, GeneralizationError> {
        let GeneralizedTriple(s, p, o) = self;
        let s = local_term_to_id(s).map_err(|_| GeneralizationError::LiteralSubject)?;
        let p = match p {
            LocalTerm::BlankId(_) => return Err(GeneralizationError::BlankPredicate),
            LocalTerm::Named(Term::Iri(iri)) => iri,
            LocalTerm::Named(Term::Literal(_)) => return Err(GeneralizationError::LiteralPredicate),
        };
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

/// Generalized RDF quad (RDF 1.1 §A) — no position bounds on its components.
#[derive(Clone, Copy, Eq, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeneralizedQuad<S = LocalTerm, P = S, O = S, G = S>(pub S, pub P, pub O, pub Option<G>);

impl<S, P, O, G> GeneralizedQuad<S, P, O, G> {
    pub const fn new(subject: S, predicate: P, object: O, graph: Option<G>) -> Self {
        Self(subject, predicate, object, graph)
    }

    pub const fn subject(&self) -> &S {
        &self.0
    }

    pub const fn predicate(&self) -> &P {
        &self.1
    }

    pub const fn object(&self) -> &O {
        &self.2
    }

    pub const fn graph(&self) -> Option<&G> {
        self.3.as_ref()
    }

    pub fn into_subject(self) -> S {
        self.0
    }

    pub fn into_predicate(self) -> P {
        self.1
    }

    pub fn into_object(self) -> O {
        self.2
    }

    pub fn into_graph(self) -> Option<G> {
        self.3
    }

    pub fn into_parts(self) -> (S, P, O, Option<G>) {
        (self.0, self.1, self.2, self.3)
    }

    pub fn into_triple(self) -> (GeneralizedTriple<S, P, O>, Option<G>) {
        (GeneralizedTriple(self.0, self.1, self.2), self.3)
    }

    pub fn map_subject<U>(self, f: impl FnOnce(S) -> U) -> GeneralizedQuad<U, P, O, G> {
        GeneralizedQuad(f(self.0), self.1, self.2, self.3)
    }

    pub fn map_predicate<U>(self, f: impl FnOnce(P) -> U) -> GeneralizedQuad<S, U, O, G> {
        GeneralizedQuad(self.0, f(self.1), self.2, self.3)
    }

    pub fn map_object<U>(self, f: impl FnOnce(O) -> U) -> GeneralizedQuad<S, P, U, G> {
        GeneralizedQuad(self.0, self.1, f(self.2), self.3)
    }

    pub fn map_graph<U>(self, f: impl FnOnce(Option<G>) -> Option<U>) -> GeneralizedQuad<S, P, O, U> {
        GeneralizedQuad(self.0, self.1, self.2, f(self.3))
    }

    pub fn with_graph(self, g: Option<G>) -> Self {
        Self(self.0, self.1, self.2, g)
    }

    pub fn map_all<S2, P2, O2, G2>(
        self,
        s: impl FnOnce(S) -> S2,
        p: impl FnOnce(P) -> P2,
        o: impl FnOnce(O) -> O2,
        g: impl FnOnce(Option<G>) -> Option<G2>,
    ) -> GeneralizedQuad<S2, P2, O2, G2> {
        GeneralizedQuad(s(self.0), p(self.1), o(self.2), g(self.3))
    }

    pub const fn as_ref(&self) -> GeneralizedQuad<&S, &P, &O, &G> {
        GeneralizedQuad(&self.0, &self.1, &self.2, self.3.as_ref())
    }
}

impl<S, P, O, G> GeneralizedQuad<&S, &P, &O, &G> {
    pub fn cloned(&self) -> GeneralizedQuad<S, P, O, G>
    where
        S: Clone,
        P: Clone,
        O: Clone,
        G: Clone,
    {
        GeneralizedQuad(self.0.clone(), self.1.clone(), self.2.clone(), self.3.cloned())
    }

    pub fn into_cloned(self) -> GeneralizedQuad<S, P, O, G>
    where
        S: Clone,
        P: Clone,
        O: Clone,
        G: Clone,
    {
        GeneralizedQuad(self.0.clone(), self.1.clone(), self.2.clone(), self.3.cloned())
    }

    pub const fn copied(&self) -> GeneralizedQuad<S, P, O, G>
    where
        S: Copy,
        P: Copy,
        O: Copy,
        G: Copy,
    {
        GeneralizedQuad(*self.0, *self.1, *self.2, self.3.copied())
    }

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
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> GeneralizedQuad<U, U, U, U> {
        GeneralizedQuad(f(self.0), f(self.1), f(self.2), self.3.map(f))
    }
}

impl GeneralizedQuad<LocalTerm, LocalTerm, LocalTerm, LocalTerm> {
    /// Tries to narrow this generalized quad into a strict [`Quad`].
    pub fn try_into_quad(self) -> Result<Quad<Id, IriBuf, LocalTerm, Id>, GeneralizationError> {
        let GeneralizedQuad(s, p, o, g) = self;
        let s = local_term_to_id(s).map_err(|_| GeneralizationError::LiteralSubject)?;
        let p = match p {
            LocalTerm::BlankId(_) => return Err(GeneralizationError::BlankPredicate),
            LocalTerm::Named(Term::Iri(iri)) => iri,
            LocalTerm::Named(Term::Literal(_)) => return Err(GeneralizationError::LiteralPredicate),
        };
        let g = match g {
            Some(t) => Some(local_term_to_id(t).map_err(|_| GeneralizationError::LiteralGraph)?),
            None => None,
        };
        Ok(Quad(s, p, o, g))
    }
}

fn local_term_to_id(t: LocalTerm) -> Result<Id, ()> {
    match t {
        LocalTerm::BlankId(b) => Ok(Id::BlankId(b)),
        LocalTerm::Named(Term::Iri(iri)) => Ok(Id::Iri(iri)),
        LocalTerm::Named(Term::Literal(_)) => Err(()),
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
