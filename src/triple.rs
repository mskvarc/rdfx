use crate::{Id, IdRef, IsGraph, IsObject, IsPredicate, IsSubject, LocalTerm, LocalTermRef, Quad, RdfDisplay};
use iri_rs::{Iri, IriBuf};
use std::{cmp::Ordering, fmt};

/// Lexical RDF triple.
pub type LexicalTriple = Triple<Id, IriBuf, LocalTerm>;

/// Lexical RDF triple reference.
pub type LexicalTripleRef<'a> = Triple<IdRef<'a>, Iri<&'a str>, LocalTermRef<'a>>;

/// RDF triple.
///
/// Position bounds enforce the RDF 1.1 abstract syntax: subjects must be IRIs
/// or blank nodes (no literals), predicates must be IRIs (no blank nodes or
/// literals), objects may be any term. For generalized RDF (RDF 1.1 §A) use
/// [`GeneralizedTriple`](crate::GeneralizedTriple).
#[derive(Clone, Copy, Eq, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Triple<S: IsSubject = Id, P: IsPredicate = IriBuf, O: IsObject = LocalTerm>(pub S, pub P, pub O);

impl<S1, P1, O1, S2, P2, O2> PartialEq<Triple<S2, P2, O2>> for Triple<S1, P1, O1>
where
    S1: IsSubject + PartialEq<S2>,
    P1: IsPredicate + PartialEq<P2>,
    O1: IsObject + PartialEq<O2>,
    S2: IsSubject,
    P2: IsPredicate,
    O2: IsObject,
{
    fn eq(&self, other: &Triple<S2, P2, O2>) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl<S1, P1, O1, S2, P2, O2> PartialOrd<Triple<S2, P2, O2>> for Triple<S1, P1, O1>
where
    S1: IsSubject + PartialOrd<S2>,
    P1: IsPredicate + PartialOrd<P2>,
    O1: IsObject + PartialOrd<O2>,
    S2: IsSubject,
    P2: IsPredicate,
    O2: IsObject,
{
    fn partial_cmp(&self, other: &Triple<S2, P2, O2>) -> Option<Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(Ordering::Equal) => match self.1.partial_cmp(&other.1) {
                Some(Ordering::Equal) => self.2.partial_cmp(&other.2),
                cmp => cmp,
            },
            cmp => cmp,
        }
    }
}

impl<S: IsSubject, P: IsPredicate, O: IsObject> Triple<S, P, O> {
    /// Creates a new triple.
    pub const fn new(subject: S, predicate: P, object: O) -> Self {
        Self(subject, predicate, object)
    }

    /// Returns a reference to the subject of the triple,
    /// the first component.
    pub const fn subject(&self) -> &S {
        &self.0
    }

    /// Returns a mutable reference to the subject of the triple,
    /// the first component.
    pub const fn subject_mut(&mut self) -> &mut S {
        &mut self.0
    }

    /// Turns the triple into its subject,
    /// the first component.
    pub fn into_subject(self) -> S {
        self.0
    }

    /// Returns a reference to the predicate of the triple,
    /// the second component.
    pub const fn predicate(&self) -> &P {
        &self.1
    }

    /// Returns a mutable reference to the predicate of the triple,
    /// the second component.
    pub const fn predicate_mut(&mut self) -> &mut P {
        &mut self.1
    }

    /// Turns the triple into its predicate,
    /// the second component.
    pub fn into_predicate(self) -> P {
        self.1
    }

    /// Returns a reference to the object of the triple,
    /// the third component.
    pub const fn object(&self) -> &O {
        &self.2
    }

    /// Returns a mutable reference to the object of the triple,
    /// the third component.
    pub const fn object_mut(&mut self) -> &mut O {
        &mut self.2
    }

    /// Turns the triple into its object,
    /// the third component.
    pub fn into_object(self) -> O {
        self.2
    }

    /// Turns the triple into a tuple
    pub fn into_parts(self) -> (S, P, O) {
        (self.0, self.1, self.2)
    }

    /// Turns the triple into a quad with the given `graph` component.
    pub fn into_quad<G: IsGraph>(self, graph: Option<G>) -> Quad<S, P, O, G> {
        Quad(self.0, self.1, self.2, graph)
    }

    /// Maps the subject with the given function.
    pub fn map_subject<U: IsSubject>(self, f: impl FnOnce(S) -> U) -> Triple<U, P, O> {
        Triple(f(self.0), self.1, self.2)
    }

    /// Maps the predicate with the given function.
    pub fn map_predicate<U: IsPredicate>(self, f: impl FnOnce(P) -> U) -> Triple<S, U, O> {
        Triple(self.0, f(self.1), self.2)
    }

    /// Maps the object with the given function.
    pub fn map_object<U: IsObject>(self, f: impl FnOnce(O) -> U) -> Triple<S, P, U> {
        Triple(self.0, self.1, f(self.2))
    }

    /// Borrows each component of the triple.
    pub const fn as_ref(&self) -> Triple<&S, &P, &O> {
        Triple(&self.0, &self.1, &self.2)
    }
}

impl<S: IsSubject, P: IsPredicate, O: IsObject> Triple<&S, &P, &O> {
    pub fn cloned(&self) -> Triple<S, P, O>
    where
        S: Clone,
        P: Clone,
        O: Clone,
    {
        Triple(self.0.clone(), self.1.clone(), self.2.clone())
    }

    pub fn into_cloned(self) -> Triple<S, P, O>
    where
        S: Clone,
        P: Clone,
        O: Clone,
    {
        Triple(self.0.clone(), self.1.clone(), self.2.clone())
    }
}

impl<S: IsSubject, P: IsPredicate, O: IsObject> Triple<&S, &P, &O> {
    pub const fn copied(&self) -> Triple<S, P, O>
    where
        S: Copy,
        P: Copy,
        O: Copy,
    {
        Triple(*self.0, *self.1, *self.2)
    }

    pub const fn into_copied(self) -> Triple<S, P, O>
    where
        S: Copy,
        P: Copy,
        O: Copy,
    {
        Triple(*self.0, *self.1, *self.2)
    }
}

impl<T: IsSubject + IsPredicate + IsObject> Triple<T, T, T> {
    /// Maps the components with the given function.
    pub fn map<U: IsSubject + IsPredicate + IsObject>(self, mut f: impl FnMut(T) -> U) -> Triple<U, U, U> {
        Triple(f(self.0), f(self.1), f(self.2))
    }
}

impl LexicalTriple {
    pub fn as_lexical_triple_ref(&self) -> LexicalTripleRef<'_> {
        Triple(self.0.as_ref(), self.1.as_ref(), self.2.as_ref())
    }
}

impl LexicalTripleRef<'_> {
    pub fn into_owned(self) -> LexicalTriple {
        Triple(self.0.into_owned(), self.1.into(), self.2.to_owned())
    }
}

impl<S, P, O> fmt::Display for Triple<S, P, O>
where
    S: IsSubject + RdfDisplay,
    P: IsPredicate + RdfDisplay,
    O: IsObject + RdfDisplay,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.0.rdf_display(), self.1.rdf_display(), self.2.rdf_display())
    }
}

impl<S, P, O> RdfDisplay for Triple<S, P, O>
where
    S: IsSubject + RdfDisplay,
    P: IsPredicate + RdfDisplay,
    O: IsObject + RdfDisplay,
{
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.0.rdf_display(), self.1.rdf_display(), self.2.rdf_display())
    }
}
