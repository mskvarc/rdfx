use core::fmt;
use std::{cmp::Ordering, fmt::Write, ops::Deref};

use educe::Educe;
use iri_rs::Iri;
use langtag::LangTag;

use crate::{Datatype, RdfDisplay, XSD_STRING};

use super::LiteralType;

/// RDF literal type reference.
///
/// Read-only counterpart of [`LiteralType`]. The `Any` variant carries no
/// runtime `rdf:langString` invariant: any reference produced via
/// [`LiteralType::as_ref`] reflects an already-validated datatype, but a
/// `LiteralTypeRef::Any` constructed directly by the user from
/// `rdf:langString` will round-trip through [`Self::into_owned`] without
/// rejection. Prefer obtaining references from validated owned values.
#[derive(Educe, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[educe(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LiteralTypeRef<'a> {
    /// Typed literal datatype IRI.
    Any(Iri<&'a str>),

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
            Self::Any(i) => i.as_str() == iri.as_str(),
            Self::LangString(_) => false,
        }
    }
}

impl LiteralTypeRef<'_> {
    pub fn into_owned(self) -> LiteralType {
        match self {
            // Safety: source assumed to come from a validated `LiteralType`.
            // A user-constructed `LiteralTypeRef::Any(rdf:langString)` will
            // round-trip an invalid value here; see type-level docs.
            Self::Any(i) => LiteralType::Any(unsafe { Datatype::new_unchecked(i.into()) }),
            Self::LangString(l) => LiteralType::LangString(l.to_owned()),
        }
    }
}

impl PartialEq<LiteralType> for LiteralTypeRef<'_> {
    fn eq(&self, other: &LiteralType) -> bool {
        match (*self, other) {
            (Self::Any(a), LiteralType::Any(b)) => a.as_str() == b.as_iri().as_str(),
            (Self::LangString(a), LiteralType::LangString(b)) => a == b.as_lang_tag(),
            _ => false,
        }
    }
}

impl<'a> PartialEq<LiteralTypeRef<'a>> for LiteralType {
    fn eq(&self, other: &LiteralTypeRef<'a>) -> bool {
        match (self, other) {
            (Self::Any(a), LiteralTypeRef::Any(b)) => a.as_iri().as_str() == b.as_str(),
            (Self::LangString(a), LiteralTypeRef::LangString(b)) => a == *b,
            _ => false,
        }
    }
}

impl PartialOrd<LiteralType> for LiteralTypeRef<'_> {
    fn partial_cmp(&self, other: &LiteralType) -> Option<Ordering> {
        match (self, other) {
            (Self::Any(a), LiteralType::Any(b)) => a.as_str().partial_cmp(b.as_iri().as_str()),
            (Self::Any(_), LiteralType::LangString(_)) => Some(Ordering::Less),
            (Self::LangString(_), LiteralType::Any(_)) => Some(Ordering::Greater),
            (Self::LangString(a), LiteralType::LangString(b)) => (*a).partial_cmp(b),
        }
    }
}

impl<'a> PartialOrd<LiteralTypeRef<'a>> for LiteralType {
    fn partial_cmp(&self, other: &LiteralTypeRef<'a>) -> Option<Ordering> {
        match (self, other) {
            (Self::Any(a), LiteralTypeRef::Any(b)) => a.as_iri().as_str().partial_cmp(b.as_str()),
            (Self::Any(_), LiteralTypeRef::LangString(_)) => Some(Ordering::Less),
            (Self::LangString(_), LiteralTypeRef::Any(_)) => Some(Ordering::Greater),
            (Self::LangString(a), LiteralTypeRef::LangString(b)) => (*a).partial_cmp(b),
        }
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
