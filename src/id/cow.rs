use std::{borrow::Cow, fmt, hash::Hash};

use iri_rs::{Iri, IriBuf};

use crate::{BlankId, BlankIdBuf, RdfDisplay};

use super::{Id, IdRef};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CowId<'a> {
    BlankId(Cow<'a, BlankId>),
    Iri(Cow<'a, IriBuf>),
}

impl<'a> CowId<'a> {
    pub fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    pub fn is_blank_id(&self) -> bool {
        matches!(self, Self::BlankId(_))
    }

    pub fn as_iri(&self) -> Option<Iri<&str>> {
        match self {
            Self::Iri(iri) => Some((**iri).as_ref()),
            Self::BlankId(_) => None,
        }
    }

    pub fn as_blank_id(&self) -> Option<&BlankId> {
        match self {
            Self::BlankId(b) => Some(b),
            Self::Iri(_) => None,
        }
    }

    pub fn as_ref(&self) -> IdRef<'_> {
        match self {
            Self::BlankId(blank_id) => IdRef::BlankId(blank_id),
            Self::Iri(iri) => IdRef::Iri((**iri).as_ref()),
        }
    }

    pub fn into_owned(self) -> Id {
        match self {
            Self::BlankId(blank_id) => Id::BlankId(blank_id.into_owned()),
            Self::Iri(iri) => Id::Iri(iri.into_owned()),
        }
    }
}

impl<'a> From<&'a BlankId> for CowId<'a> {
    fn from(value: &'a BlankId) -> Self {
        Self::BlankId(Cow::Borrowed(value))
    }
}

impl From<BlankIdBuf> for CowId<'_> {
    fn from(value: BlankIdBuf) -> Self {
        Self::BlankId(Cow::Owned(value))
    }
}

impl From<IriBuf> for CowId<'_> {
    fn from(value: IriBuf) -> Self {
        Self::Iri(Cow::Owned(value))
    }
}

impl Hash for CowId<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::BlankId(id) => id.hash(state),
            Self::Iri(iri) => iri.hash(state),
        }
    }
}

impl fmt::Display for CowId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(id) => id.fmt(f),
            Self::Iri(iri) => iri.fmt(f),
        }
    }
}

impl RdfDisplay for CowId<'_> {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(id) => id.rdf_fmt(f),
            Self::Iri(iri) => iri.rdf_fmt(f),
        }
    }
}
