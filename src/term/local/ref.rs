use crate::{BlankId, RdfDisplay, TermRef, TripleTerm};
use std::{cmp::Ordering, fmt, sync::Arc};

use super::LocalTerm;

/// Lexical RDF term reference.
///
/// The [`LocalTermRef::Triple`] variant carries a `&TripleTerm` rather than a
/// fully borrowed inner triple — borrowing always points at the owned
/// triple-term body inside [`LocalTerm::Triple`], avoiding intermediate
/// heap allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LocalTermRef<'a> {
    BlankId(&'a BlankId),

    Named(TermRef<'a>),

    /// RDF 1.2 triple term — borrowed.
    Triple(&'a TripleTerm),
}

impl<'a> LocalTermRef<'a> {
    pub fn to_owned(self) -> LocalTerm {
        match self {
            Self::BlankId(blank_id) => LocalTerm::BlankId(blank_id.to_owned()),
            Self::Named(named) => LocalTerm::Named(named.to_owned()),
            Self::Triple(t) => LocalTerm::Triple(Arc::new(t.clone())),
        }
    }

    pub const fn is_triple(&self) -> bool {
        matches!(self, Self::Triple(_))
    }

    pub const fn as_triple(self) -> Option<&'a TripleTerm> {
        match self {
            Self::Triple(t) => Some(t),
            _ => None,
        }
    }
}

impl PartialEq<LocalTerm> for LocalTermRef<'_> {
    fn eq(&self, other: &LocalTerm) -> bool {
        match (self, other) {
            (Self::BlankId(a), LocalTerm::BlankId(b)) => *a == b,
            (Self::Named(a), LocalTerm::Named(b)) => a == b,
            (Self::Triple(a), LocalTerm::Triple(b)) => **a == **b,
            _ => false,
        }
    }
}

impl PartialOrd<LocalTerm> for LocalTermRef<'_> {
    fn partial_cmp(&self, other: &LocalTerm) -> Option<Ordering> {
        match (self, other) {
            (Self::BlankId(a), LocalTerm::BlankId(b)) => (*a).partial_cmp(b),
            (Self::BlankId(_), _) => Some(Ordering::Less),
            (_, LocalTerm::BlankId(_)) => Some(Ordering::Greater),
            (Self::Named(a), LocalTerm::Named(b)) => a.partial_cmp(b),
            (Self::Named(_), LocalTerm::Triple(_)) => Some(Ordering::Less),
            (Self::Triple(_), LocalTerm::Named(_)) => Some(Ordering::Greater),
            (Self::Triple(a), LocalTerm::Triple(b)) => (**a).partial_cmp(&**b),
        }
    }
}

impl fmt::Display for LocalTermRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(id) => id.fmt(f),
            Self::Named(t) => t.fmt(f),
            Self::Triple(t) => {
                f.write_str("<<( ")?;
                t.rdf_fmt(f)?;
                f.write_str(" )>>")
            }
        }
    }
}

impl RdfDisplay for LocalTermRef<'_> {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(id) => id.rdf_fmt(f),
            Self::Named(t) => t.rdf_fmt(f),
            Self::Triple(t) => {
                f.write_str("<<( ")?;
                t.rdf_fmt(f)?;
                f.write_str(" )>>")
            }
        }
    }
}
