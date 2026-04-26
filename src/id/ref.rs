use std::fmt;

use iri_rs::Iri;

use crate::{BlankId, RdfDisplay};

use super::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdRef<'a> {
    BlankId(&'a BlankId),
    Iri(Iri<&'a str>),
}

impl IdRef<'_> {
    pub fn to_owned(&self) -> Id {
        self.into_owned()
    }

    pub fn into_owned(self) -> Id {
        match self {
            Self::BlankId(blank_id) => Id::BlankId(blank_id.to_owned()),
            Self::Iri(iri) => Id::Iri(iri.into()),
        }
    }
}

impl fmt::Display for IdRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(b) => b.fmt(f),
            Self::Iri(iri) => iri.fmt(f),
        }
    }
}

impl RdfDisplay for IdRef<'_> {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BlankId(b) => b.rdf_fmt(f),
            Self::Iri(iri) => iri.rdf_fmt(f),
        }
    }
}
