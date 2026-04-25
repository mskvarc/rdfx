use core::fmt;
use std::{cmp::Ordering, fmt::Write, ops::Deref};

use educe::Educe;
use iri_rs::Iri;
use langtag::LangTag;

use crate::{RdfDisplay, XSD_STRING};

use crate::DatatypeRef;

use super::LiteralType;

/// RDF literal type reference.
///
/// Read-only counterpart of [`LiteralType`]. The `Any` variant wraps a
/// validated [`DatatypeRef`] so safe construction cannot produce a
/// `LiteralTypeRef::Any(rdf:langString)`.
#[derive(Educe, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[educe(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LiteralTypeRef<'a> {
    /// Typed literal datatype.
    Any(DatatypeRef<'a>),

    /// Language-tagged literal.
    LangString(&'a LangTag),
}

impl<'a> LiteralTypeRef<'a> {
    pub const fn is_lang_string(&self) -> bool {
        matches!(self, Self::LangString(_))
    }

    pub const fn lang_tag(&self) -> Option<&'a LangTag> {
        match self {
            Self::LangString(tag) => Some(tag),
            _ => None,
        }
    }

    pub fn is_xsd_string(&self) -> bool {
        self.is_iri(&XSD_STRING)
    }

    pub fn is_iri<T: Deref<Target = str>>(&self, iri: &Iri<T>) -> bool {
        match self {
            Self::Any(d) => d.as_iri().as_str() == iri.as_str(),
            Self::LangString(_) => false,
        }
    }
}

impl LiteralTypeRef<'_> {
    pub fn into_owned(self) -> LiteralType {
        match self {
            Self::Any(d) => LiteralType::Any(d.to_owned()),
            Self::LangString(l) => LiteralType::LangString(l.to_owned()),
        }
    }
}

impl PartialEq<LiteralType> for LiteralTypeRef<'_> {
    fn eq(&self, other: &LiteralType) -> bool {
        match (*self, other) {
            (Self::Any(a), LiteralType::Any(b)) => a.as_iri().as_str() == b.as_iri().as_str(),
            (Self::LangString(a), LiteralType::LangString(b)) => a == b.as_lang_tag(),
            _ => false,
        }
    }
}

impl<'a> PartialEq<LiteralTypeRef<'a>> for LiteralType {
    fn eq(&self, other: &LiteralTypeRef<'a>) -> bool {
        match (self, other) {
            (Self::Any(a), LiteralTypeRef::Any(b)) => a.as_iri().as_str() == b.as_iri().as_str(),
            (Self::LangString(a), LiteralTypeRef::LangString(b)) => a == *b,
            _ => false,
        }
    }
}

impl PartialOrd<LiteralType> for LiteralTypeRef<'_> {
    fn partial_cmp(&self, other: &LiteralType) -> Option<Ordering> {
        match (self, other) {
            (Self::Any(a), LiteralType::Any(b)) => a.as_iri().as_str().partial_cmp(b.as_iri().as_str()),
            (Self::Any(_), LiteralType::LangString(_)) => Some(Ordering::Less),
            (Self::LangString(_), LiteralType::Any(_)) => Some(Ordering::Greater),
            (Self::LangString(a), LiteralType::LangString(b)) => (*a).partial_cmp(b),
        }
    }
}

impl<'a> PartialOrd<LiteralTypeRef<'a>> for LiteralType {
    fn partial_cmp(&self, other: &LiteralTypeRef<'a>) -> Option<Ordering> {
        match (self, other) {
            (Self::Any(a), LiteralTypeRef::Any(b)) => a.as_iri().as_str().partial_cmp(b.as_iri().as_str()),
            (Self::Any(_), LiteralTypeRef::LangString(_)) => Some(Ordering::Less),
            (Self::LangString(_), LiteralTypeRef::Any(_)) => Some(Ordering::Greater),
            (Self::LangString(a), LiteralTypeRef::LangString(b)) => (*a).partial_cmp(b),
        }
    }
}

impl<'a> From<DatatypeRef<'a>> for LiteralTypeRef<'a> {
    fn from(value: DatatypeRef<'a>) -> Self {
        Self::Any(value)
    }
}

impl<'a> From<&'a LangTag> for LiteralTypeRef<'a> {
    fn from(value: &'a LangTag) -> Self {
        Self::LangString(value)
    }
}

impl RdfDisplay for LiteralTypeRef<'_> {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Any(ty) => {
                f.write_str("^^")?;
                ty.rdf_fmt(f)
            }
            Self::LangString(tag) => {
                f.write_char('@')?;
                tag.rdf_fmt(f)
            }
        }
    }
}

