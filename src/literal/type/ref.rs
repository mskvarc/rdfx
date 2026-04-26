use core::fmt;
use std::{cmp::Ordering, fmt::Write, ops::Deref};

use educe::Educe;
use iri_rs::Iri;
use langtag::LangTag;

use crate::{Direction, RdfDisplay, XSD_STRING};

use crate::DatatypeRef;

use super::LiteralType;

/// RDF literal type reference.
///
/// Read-only counterpart of [`LiteralType`]. The `Any` variant wraps a
/// validated [`DatatypeRef`] so safe construction cannot produce a
/// `LiteralTypeRef::Any(rdf:langString)` or
/// `LiteralTypeRef::Any(rdf:dirLangString)`.
#[derive(Educe, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[educe(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LiteralTypeRef<'a> {
    /// Typed literal datatype.
    Any(DatatypeRef<'a>),

    /// Language-tagged literal with no base direction.
    LangString(&'a LangTag),

    /// Directional language-tagged literal (RDF 1.2).
    DirLangString {
        /// Language tag.
        tag: &'a LangTag,
        /// Base direction.
        direction: Direction,
    },
}

impl<'a> LiteralTypeRef<'a> {
    /// Returns `true` for both [`Self::LangString`] and [`Self::DirLangString`].
    pub const fn is_lang_string(&self) -> bool {
        matches!(self, Self::LangString(_) | Self::DirLangString { .. })
    }

    /// Returns `true` only for [`Self::LangString`] (no direction).
    pub const fn is_undirected_lang_string(&self) -> bool {
        matches!(self, Self::LangString(_))
    }

    /// Returns `true` for [`Self::DirLangString`] (RDF 1.2).
    pub const fn is_dir_lang_string(&self) -> bool {
        matches!(self, Self::DirLangString { .. })
    }

    pub const fn lang_tag(&self) -> Option<&'a LangTag> {
        match self {
            Self::LangString(tag) => Some(tag),
            Self::DirLangString { tag, .. } => Some(tag),
            Self::Any(_) => None,
        }
    }

    /// Returns the base direction for [`Self::DirLangString`] only.
    pub const fn direction(&self) -> Option<Direction> {
        match self {
            Self::DirLangString { direction, .. } => Some(*direction),
            _ => None,
        }
    }

    pub fn is_xsd_string(&self) -> bool {
        self.is_iri(&XSD_STRING)
    }

    pub fn is_iri<T: Deref<Target = str>>(&self, iri: &Iri<T>) -> bool {
        match self {
            Self::Any(d) => d.as_iri().as_str() == iri.as_str(),
            Self::LangString(_) | Self::DirLangString { .. } => false,
        }
    }
}

impl LiteralTypeRef<'_> {
    pub fn into_owned(self) -> LiteralType {
        match self {
            Self::Any(d) => LiteralType::Any(d.to_owned()),
            Self::LangString(l) => LiteralType::LangString(l.to_owned()),
            Self::DirLangString { tag, direction } => LiteralType::DirLangString {
                tag: tag.to_owned(),
                direction,
            },
        }
    }
}

impl PartialEq<LiteralType> for LiteralTypeRef<'_> {
    fn eq(&self, other: &LiteralType) -> bool {
        match (*self, other) {
            (Self::Any(a), LiteralType::Any(b)) => a.as_iri().as_str() == b.as_iri().as_str(),
            (Self::LangString(a), LiteralType::LangString(b)) => a == b.as_lang_tag(),
            (Self::DirLangString { tag: a, direction: da }, LiteralType::DirLangString { tag: b, direction: db }) => a == b.as_lang_tag() && da == *db,
            _ => false,
        }
    }
}

impl<'a> PartialEq<LiteralTypeRef<'a>> for LiteralType {
    fn eq(&self, other: &LiteralTypeRef<'a>) -> bool {
        match (self, other) {
            (Self::Any(a), LiteralTypeRef::Any(b)) => a.as_iri().as_str() == b.as_iri().as_str(),
            (Self::LangString(a), LiteralTypeRef::LangString(b)) => a == *b,
            (Self::DirLangString { tag: a, direction: da }, LiteralTypeRef::DirLangString { tag: b, direction: db }) => a == *b && da == db,
            _ => false,
        }
    }
}

impl PartialOrd<LiteralType> for LiteralTypeRef<'_> {
    fn partial_cmp(&self, other: &LiteralType) -> Option<Ordering> {
        match (self, other) {
            (Self::Any(a), LiteralType::Any(b)) => a.as_iri().as_str().partial_cmp(b.as_iri().as_str()),
            (Self::Any(_), _) => Some(Ordering::Less),
            (_, LiteralType::Any(_)) => Some(Ordering::Greater),
            (Self::LangString(a), LiteralType::LangString(b)) => (*a).partial_cmp(b),
            (Self::LangString(_), LiteralType::DirLangString { .. }) => Some(Ordering::Less),
            (Self::DirLangString { .. }, LiteralType::LangString(_)) => Some(Ordering::Greater),
            (Self::DirLangString { tag: a, direction: da }, LiteralType::DirLangString { tag: b, direction: db }) => match (*a).partial_cmp(b) {
                Some(Ordering::Equal) => da.partial_cmp(db),
                ord => ord,
            },
        }
    }
}

impl<'a> PartialOrd<LiteralTypeRef<'a>> for LiteralType {
    fn partial_cmp(&self, other: &LiteralTypeRef<'a>) -> Option<Ordering> {
        match (self, other) {
            (Self::Any(a), LiteralTypeRef::Any(b)) => a.as_iri().as_str().partial_cmp(b.as_iri().as_str()),
            (Self::Any(_), _) => Some(Ordering::Less),
            (_, LiteralTypeRef::Any(_)) => Some(Ordering::Greater),
            (Self::LangString(a), LiteralTypeRef::LangString(b)) => (*a).partial_cmp(b),
            (Self::LangString(_), LiteralTypeRef::DirLangString { .. }) => Some(Ordering::Less),
            (Self::DirLangString { .. }, LiteralTypeRef::LangString(_)) => Some(Ordering::Greater),
            (Self::DirLangString { tag: a, direction: da }, LiteralTypeRef::DirLangString { tag: b, direction: db }) => match (*a).partial_cmp(b) {
                Some(Ordering::Equal) => da.partial_cmp(db),
                ord => ord,
            },
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
            Self::DirLangString { tag, direction } => {
                f.write_char('@')?;
                tag.rdf_fmt(f)?;
                f.write_str("--")?;
                f.write_str(direction.as_str())
            }
        }
    }
}
