use std::borrow::Cow;

use crate::BlankIdBuf;
use iri_rs::{Iri, IriBuf};

mod r#ref;
pub use r#ref::*;

mod cow;
pub use cow::*;

pub enum Id {
    BlankId(BlankIdBuf),
    Iri(IriBuf),
}

impl Id {
    pub fn as_iri(&self) -> Option<Iri<&str>> {
        match self {
            Self::Iri(iri) => Some(iri.as_ref()),
            _ => None,
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
}
