use std::borrow::Cow;

use educe::Educe;
use iri_rs::{Iri, IriBuf};
use langtag::{LangTag, LangTagBuf};

use super::{LiteralType, LiteralTypeRef};

/// Owned or referenced RDF literal type.
#[derive(Educe, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[educe(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum CowLiteralType<'a> {
    /// Any type.
    Any(Cow<'a, IriBuf>),

    /// Language string.
    LangString(Cow<'a, LangTag>),
}

impl CowLiteralType<'_> {
    pub fn as_ref(&self) -> LiteralTypeRef<'_> {
        match self {
            Self::Any(i) => LiteralTypeRef::Any(i.as_ref().into()),
            Self::LangString(l) => LiteralTypeRef::LangString(l),
        }
    }

    pub fn into_owned(self) -> LiteralType {
        match self {
            Self::Any(iri) => LiteralType::Any(iri.into_owned()),
            Self::LangString(lang) => LiteralType::LangString(lang.into_owned()),
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
            LiteralTypeRef::Any(i) => Self::Any(Cow::Owned(i.into())),
            LiteralTypeRef::LangString(t) => Self::LangString(Cow::Borrowed(t)),
        }
    }
}

impl<'a> From<Iri<&'a str>> for CowLiteralType<'a> {
    fn from(value: Iri<&'a str>) -> Self {
        Self::Any(Cow::Owned(value.into()))
    }
}

impl<'a> From<&'a IriBuf> for CowLiteralType<'a> {
    fn from(value: &'a IriBuf) -> Self {
        Self::Any(Cow::Borrowed(value))
    }
}

impl From<IriBuf> for CowLiteralType<'_> {
    fn from(value: IriBuf) -> Self {
        Self::Any(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, IriBuf>> for CowLiteralType<'a> {
    fn from(value: Cow<'a, IriBuf>) -> Self {
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
