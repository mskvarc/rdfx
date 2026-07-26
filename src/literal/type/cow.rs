// Educe's `Clone` derive on a struct-style variant emits `_x`-prefixed
// shadow bindings whose spans point back to the field declarations,
// triggering false-positive `used_underscore_binding` warnings.
#![allow(clippy::used_underscore_binding)]

use std::borrow::Cow;

use educe::Educe;
use iri_rs::{Iri, IriBuf};
use langtag::{LangTag, LangTagBuf};

use crate::{Direction, RDF_DIR_LANG_STRING, RDF_LANG_STRING};

use super::{Datatype, LiteralType, LiteralTypeRef, ReservedDatatypeError};

/// Owned or referenced RDF literal type.
#[derive(Educe, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[educe(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum CowLiteralType<'a> {
    /// Typed literal datatype.
    Any(Cow<'a, Datatype>),

    /// Language-tagged literal with no base direction.
    LangString(Cow<'a, LangTag>),

    /// Directional language-tagged literal (RDF 1.2).
    DirLangString {
        /// Language tag.
        tag: Cow<'a, LangTag>,
        /// Base direction.
        direction: Direction,
    },
}

impl CowLiteralType<'_> {
    pub fn as_ref(&self) -> LiteralTypeRef<'_> {
        match self {
            Self::Any(d) => LiteralTypeRef::Any(d.as_datatype_ref()),
            Self::LangString(l) => LiteralTypeRef::LangString(l),
            Self::DirLangString { tag, direction } => LiteralTypeRef::DirLangString { tag, direction: *direction },
        }
    }

    pub fn into_owned(self) -> LiteralType {
        match self {
            Self::Any(d) => LiteralType::Any(d.into_owned()),
            Self::LangString(lang) => LiteralType::LangString(lang.into_owned()),
            Self::DirLangString { tag, direction } => LiteralType::DirLangString {
                tag: tag.into_owned(),
                direction,
            },
        }
    }
}

impl<'a> From<CowLiteralType<'a>> for LiteralType {
    fn from(value: CowLiteralType<'a>) -> Self {
        value.into_owned()
    }
}

impl<'a> From<LiteralTypeRef<'a>> for CowLiteralType<'a> {
    fn from(value: LiteralTypeRef<'a>) -> Self {
        match value {
            LiteralTypeRef::Any(d) => Self::Any(Cow::Owned(d.to_owned())),
            LiteralTypeRef::LangString(t) => Self::LangString(Cow::Borrowed(t)),
            LiteralTypeRef::DirLangString { tag, direction } => Self::DirLangString {
                tag: Cow::Borrowed(tag),
                direction,
            },
        }
    }
}

impl<'a> From<&'a Datatype> for CowLiteralType<'a> {
    fn from(value: &'a Datatype) -> Self {
        Self::Any(Cow::Borrowed(value))
    }
}

impl From<Datatype> for CowLiteralType<'_> {
    fn from(value: Datatype) -> Self {
        Self::Any(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, Datatype>> for CowLiteralType<'a> {
    fn from(value: Cow<'a, Datatype>) -> Self {
        Self::Any(value)
    }
}

impl<'a> From<&'a LangTag> for CowLiteralType<'a> {
    fn from(value: &'a LangTag) -> Self {
        Self::LangString(Cow::Borrowed(value))
    }
}

impl From<LangTagBuf> for CowLiteralType<'_> {
    fn from(value: LangTagBuf) -> Self {
        Self::LangString(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, LangTag>> for CowLiteralType<'a> {
    fn from(value: Cow<'a, LangTag>) -> Self {
        Self::LangString(value)
    }
}

impl TryFrom<IriBuf> for CowLiteralType<'_> {
    type Error = ReservedDatatypeError;
    fn try_from(value: IriBuf) -> Result<Self, Self::Error> {
        Datatype::new(value).map(|d| Self::Any(Cow::Owned(d)))
    }
}

impl<'a> TryFrom<Iri<&'a str>> for CowLiteralType<'a> {
    type Error = ReservedDatatypeError;
    fn try_from(value: Iri<&'a str>) -> Result<Self, Self::Error> {
        if value.as_str() == RDF_LANG_STRING.as_str() {
            Err(ReservedDatatypeError::LangString(value.into()))
        } else if value.as_str() == RDF_DIR_LANG_STRING.as_str() {
            Err(ReservedDatatypeError::DirLangString(value.into()))
        } else {
            // Safety: just checked `value` is neither reserved IRI.
            Ok(Self::Any(Cow::Owned(unsafe { Datatype::new_unchecked(value.into()) })))
        }
    }
}
