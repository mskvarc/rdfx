use crate::{BlankId, TermRef};
use std::cmp::Ordering;

use super::LocalTerm;

/// Lexical RDF term reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LocalTermRef<'a> {
    BlankId(&'a BlankId),

    Named(TermRef<'a>),
}

impl LocalTermRef<'_> {
    pub fn to_owned(self) -> LocalTerm {
        match self {
            Self::BlankId(blank_id) => LocalTerm::BlankId(blank_id.to_owned()),
            Self::Named(named) => LocalTerm::Named(named.to_owned()),
        }
    }
}

impl PartialEq<LocalTerm> for LocalTermRef<'_> {
    fn eq(&self, other: &LocalTerm) -> bool {
        match (self, other) {
            (Self::BlankId(a), LocalTerm::BlankId(b)) => *a == b,
            (Self::Named(a), LocalTerm::Named(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd<LocalTerm> for LocalTermRef<'_> {
    fn partial_cmp(&self, other: &LocalTerm) -> Option<Ordering> {
        match (self, other) {
            (Self::BlankId(a), LocalTerm::BlankId(b)) => (*a).partial_cmp(b),
            (Self::BlankId(_), LocalTerm::Named(_)) => Some(Ordering::Less),
            (Self::Named(_), LocalTerm::BlankId(_)) => Some(Ordering::Greater),
            (Self::Named(a), LocalTerm::Named(b)) => (*a).partial_cmp(b),
        }
    }
}
