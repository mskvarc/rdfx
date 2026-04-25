use std::cmp::Ordering;

use crate::LiteralRef;
use iri_rs::Iri;

use super::Term;

/// Lexical RDF term reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TermRef<'a> {
    Iri(Iri<&'a str>),

    Literal(LiteralRef<'a>),
}

impl TermRef<'_> {
    pub fn to_owned(self) -> Term {
        match self {
            Self::Iri(iri) => Term::Iri(iri.into()),
            Self::Literal(l) => Term::Literal(l.to_owned()),
        }
    }
}

impl PartialEq<Term> for TermRef<'_> {
    fn eq(&self, other: &Term) -> bool {
        match (self, other) {
            (Self::Iri(a), Term::Iri(b)) => a.as_str() == b.as_str(),
            (Self::Literal(a), Term::Literal(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd<Term> for TermRef<'_> {
    fn partial_cmp(&self, other: &Term) -> Option<Ordering> {
        match (self, other) {
            (Self::Iri(a), Term::Iri(b)) => a.as_str().partial_cmp(b.as_str()),
            (Self::Iri(_), Term::Literal(_)) => Some(Ordering::Less),
            (Self::Literal(_), Term::Iri(_)) => Some(Ordering::Greater),
            (Self::Literal(a), Term::Literal(b)) => (*a).partial_cmp(b),
        }
    }
}
