use crate::{BlankId, CowTerm, TripleTerm};
use std::borrow::Cow;

use super::LocalTerm;

pub enum CowLocalTerm<'a> {
    BlankId(Cow<'a, BlankId>),

    Named(CowTerm<'a>),

    /// RDF 1.2 triple term — owned or borrowed.
    Triple(Cow<'a, TripleTerm>),
}

impl From<LocalTerm> for CowLocalTerm<'_> {
    fn from(value: LocalTerm) -> Self {
        value.into_cow()
    }
}
