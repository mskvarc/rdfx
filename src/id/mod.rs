use std::{borrow::Cow, fmt, hash::Hash};

use crate::{BlankId, BlankIdBuf, RdfDisplay};
use iri_rs::{Iri, IriBuf};

mod r#ref;
pub use r#ref::*;

mod cow;
pub use cow::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Id {
    BlankId(BlankIdBuf),
    Iri(IriBuf),
}

impl Id {
    pub const fn iri(iri: IriBuf) -> Self {
        Self::Iri(iri)
    }

    pub const fn blank_id(blank_id: BlankIdBuf) -> Self {
        Self::BlankId(blank_id)
    }

    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    pub const fn is_blank_id(&self) -> bool {
        matches!(self, Self::BlankId(_))
    }

    pub fn as_iri(&self) -> Option<Iri<&str>> {
        match self {
            Self::Iri(iri) => Some(iri.as_ref()),
            Self::BlankId(_) => None,
        }
    }

    pub fn as_blank_id(&self) -> Option<&BlankId> {
        match self {
            Self::BlankId(b) => Some(b),
            Self::Iri(_) => None,
        }
    }

    pub fn into_iri(self) -> Option<IriBuf> {
        match self {
            Self::Iri(iri) => Some(iri),
            Self::BlankId(_) => None,
        }
    }

    pub fn into_blank_id(self) -> Option<BlankIdBuf> {
        match self {
            Self::BlankId(b) => Some(b),
            Self::Iri(_) => None,
        }
    }

    pub fn as_ref(&self) -> IdRef<'_> {
        match self {
            Self::BlankId(blank_id) => IdRef::BlankId(blank_id),
            Self::Iri(iri) => IdRef::Iri(iri.as_ref()),
        }
    }

    pub fn as_cow(&self) -> CowId<'_> {
        match self {
            Self::BlankId(blank_id) => CowId::BlankId(Cow::Borrowed(blank_id)),
            Self::Iri(iri) => CowId::Iri(Cow::Borrowed(iri)),
        }
    }

    pub fn into_cow(self) -> CowId<'static> {
        match self {
            Self::BlankId(blank_id) => CowId::BlankId(Cow::Owned(blank_id)),
            Self::Iri(iri) => CowId::Iri(Cow::Owned(iri)),
        }
    }

    pub const fn into_owned(self) -> Self {
        self
    }
}

impl From<IriBuf> for Id {
    fn from(value: IriBuf) -> Self {
        Self::Iri(value)
    }
}

impl From<BlankIdBuf> for Id {
    fn from(value: BlankIdBuf) -> Self {
        Self::BlankId(value)
    }
}

impl Hash for Id {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::BlankId(id) => {
                state.write_u8(0);
                id.hash(state);
            }
            Self::Iri(iri) => {
                state.write_u8(1);
                iri.hash(state);
            }
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(id) => id.fmt(f),
            Self::Iri(iri) => iri.fmt(f),
        }
    }
}

impl RdfDisplay for Id {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(id) => id.rdf_fmt(f),
            Self::Iri(iri) => iri.rdf_fmt(f),
        }
    }
}
