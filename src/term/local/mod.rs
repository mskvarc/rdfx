use crate::{BlankId, BlankIdBuf, Literal, LiteralRef, RdfDisplay};
use std::{borrow::Cow, fmt, hash::Hash};

mod r#ref;
use iri_rs::{Iri, IriBuf};
pub use r#ref::*;

mod cow;
pub use cow::*;

use super::Term;

/// Lexical representation of an RDF resource.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum LocalTerm {
    BlankId(BlankIdBuf),
    Named(Term),
}

impl LocalTerm {
    pub const fn iri(iri: IriBuf) -> Self {
        Self::Named(Term::Iri(iri))
    }

    pub const fn literal(literal: Literal) -> Self {
        Self::Named(Term::Literal(literal))
    }

    pub const fn is_blank_id(&self) -> bool {
        matches!(self, Self::BlankId(_))
    }

    pub fn as_blank_id(&self) -> Option<&BlankId> {
        match self {
            Self::BlankId(b) => Some(b),
            Self::Named(_) => None,
        }
    }

    pub fn as_iri(&self) -> Option<Iri<&str>> {
        match self {
            Self::Named(t) => t.as_iri(),
            Self::BlankId(_) => None,
        }
    }

    pub fn as_literal(&self) -> Option<LiteralRef<'_>> {
        match self {
            Self::Named(t) => t.as_literal(),
            Self::BlankId(_) => None,
        }
    }

    pub fn as_ref(&self) -> LocalTermRef<'_> {
        match self {
            Self::BlankId(blank_id) => LocalTermRef::BlankId(blank_id),
            Self::Named(named) => LocalTermRef::Named(named.as_ref()),
        }
    }

    pub fn as_cow(&self) -> CowLocalTerm<'_> {
        match self {
            Self::BlankId(blank_id) => CowLocalTerm::BlankId(Cow::Borrowed(blank_id)),
            Self::Named(named) => CowLocalTerm::Named(named.as_cow()),
        }
    }

    pub fn into_cow(self) -> CowLocalTerm<'static> {
        match self {
            Self::BlankId(blank_id) => CowLocalTerm::BlankId(Cow::Owned(blank_id)),
            Self::Named(named) => CowLocalTerm::Named(named.into_cow()),
        }
    }
}

impl From<IriBuf> for LocalTerm {
    fn from(value: IriBuf) -> Self {
        Self::iri(value)
    }
}

impl From<Literal> for LocalTerm {
    fn from(value: Literal) -> Self {
        Self::literal(value)
    }
}

impl From<Term> for LocalTerm {
    fn from(value: Term) -> Self {
        Self::Named(value)
    }
}

impl From<BlankIdBuf> for LocalTerm {
    fn from(value: BlankIdBuf) -> Self {
        Self::BlankId(value)
    }
}

impl Hash for LocalTerm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::BlankId(id) => id.hash(state),
            Self::Named(l) => l.hash(state),
        }
    }
}

impl fmt::Display for LocalTerm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(id) => id.fmt(f),
            Self::Named(lit) => lit.fmt(f),
        }
    }
}

impl RdfDisplay for LocalTerm {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(id) => id.rdf_fmt(f),
            Self::Named(lit) => lit.rdf_fmt(f),
        }
    }
}
