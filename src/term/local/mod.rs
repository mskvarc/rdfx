use crate::{BlankId, BlankIdBuf, Literal, LiteralRef, RdfDisplay, TripleTerm};
use std::{borrow::Cow, fmt, hash::Hash};

mod r#ref;
use iri_rs::{Iri, IriBuf};
pub use r#ref::*;

mod cow;
pub use cow::*;

use super::Term;

/// Lexical representation of an RDF resource.
///
/// Per RDF 1.2 Concepts §4 a term may also be a *triple term* — a triple
/// appearing in object position. The [`LocalTerm::Triple`] variant carries a
/// strict-RDF triple body, restricting triple terms to the object slot via
/// the position-trait machinery ([`IsObject`](crate::IsObject)).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum LocalTerm {
    BlankId(BlankIdBuf),
    Named(Term),
    /// RDF 1.2 triple term — a triple appearing as a term.
    Triple(Box<TripleTerm>),
}

impl LocalTerm {
    pub const fn iri(iri: IriBuf) -> Self {
        Self::Named(Term::Iri(iri))
    }

    pub const fn literal(literal: Literal) -> Self {
        Self::Named(Term::Literal(literal))
    }

    /// Constructs a triple-term [`LocalTerm`] (RDF 1.2).
    pub fn triple(t: TripleTerm) -> Self {
        Self::Triple(Box::new(t))
    }

    pub const fn is_blank_id(&self) -> bool {
        matches!(self, Self::BlankId(_))
    }

    /// Returns `true` for [`Self::Triple`] (RDF 1.2 triple term).
    pub const fn is_triple(&self) -> bool {
        matches!(self, Self::Triple(_))
    }

    pub fn as_blank_id(&self) -> Option<&BlankId> {
        match self {
            Self::BlankId(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_iri(&self) -> Option<Iri<&str>> {
        match self {
            Self::Named(t) => t.as_iri(),
            _ => None,
        }
    }

    pub fn as_literal(&self) -> Option<LiteralRef<'_>> {
        match self {
            Self::Named(t) => t.as_literal(),
            _ => None,
        }
    }

    /// Borrows the wrapped triple body for [`Self::Triple`].
    pub fn as_triple(&self) -> Option<&TripleTerm> {
        match self {
            Self::Triple(t) => Some(t),
            _ => None,
        }
    }

    /// Consumes self, returning the triple body for [`Self::Triple`].
    pub fn into_triple(self) -> Option<TripleTerm> {
        match self {
            Self::Triple(t) => Some(*t),
            _ => None,
        }
    }

    /// Consumes self, returning the triple body for [`Self::Triple`] or the
    /// original `LocalTerm` on mismatch.
    pub fn try_into_triple(self) -> Result<TripleTerm, Self> {
        match self {
            Self::Triple(t) => Ok(*t),
            other => Err(other),
        }
    }

    pub fn as_ref(&self) -> LocalTermRef<'_> {
        match self {
            Self::BlankId(blank_id) => LocalTermRef::BlankId(blank_id),
            Self::Named(named) => LocalTermRef::Named(named.as_ref()),
            Self::Triple(t) => LocalTermRef::Triple(t),
        }
    }

    pub fn as_cow(&self) -> CowLocalTerm<'_> {
        match self {
            Self::BlankId(blank_id) => CowLocalTerm::BlankId(Cow::Borrowed(blank_id)),
            Self::Named(named) => CowLocalTerm::Named(named.as_cow()),
            Self::Triple(t) => CowLocalTerm::Triple(Cow::Borrowed(t)),
        }
    }

    pub fn into_cow(self) -> CowLocalTerm<'static> {
        match self {
            Self::BlankId(blank_id) => CowLocalTerm::BlankId(Cow::Owned(blank_id)),
            Self::Named(named) => CowLocalTerm::Named(named.into_cow()),
            Self::Triple(t) => CowLocalTerm::Triple(Cow::Owned(*t)),
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

impl From<TripleTerm> for LocalTerm {
    fn from(value: TripleTerm) -> Self {
        Self::triple(value)
    }
}

impl Hash for LocalTerm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::BlankId(id) => {
                state.write_u8(0);
                id.hash(state);
            }
            Self::Named(l) => {
                state.write_u8(1);
                l.hash(state);
            }
            Self::Triple(t) => {
                state.write_u8(2);
                t.hash(state);
            }
        }
    }
}

impl fmt::Display for LocalTerm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(id) => id.fmt(f),
            Self::Named(lit) => lit.fmt(f),
            Self::Triple(t) => write_triple_term(f, t),
        }
    }
}

impl RdfDisplay for LocalTerm {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(id) => id.rdf_fmt(f),
            Self::Named(lit) => lit.rdf_fmt(f),
            Self::Triple(t) => write_triple_term(f, t),
        }
    }
}

fn write_triple_term(f: &mut fmt::Formatter, t: &TripleTerm) -> fmt::Result {
    f.write_str("<<( ")?;
    t.rdf_fmt(f)?;
    f.write_str(" )>>")
}
