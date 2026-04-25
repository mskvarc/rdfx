use core::fmt;
use iri_rs::{Iri, IriBuf};
use langtag::{LangTag, LangTagBuf};
use std::{borrow::Cow, fmt::Write, ops::Deref};

use crate::{Datatype, RdfDisplay, RdfLangStringDatatypeError, XSD_STRING};

mod r#ref;
pub use r#ref::*;

mod cow;
pub use cow::*;

/// RDF literal type.
///
/// Per [RDF 1.1 Concepts §3.3][s], a literal is either:
/// - a typed literal with a datatype IRI other than `rdf:langString`
///   ([`LiteralType::Any`]), or
/// - a language-tagged literal carrying a language tag
///   ([`LiteralType::LangString`]).
///
/// `LiteralType::Any` wraps a [`Datatype`] whose constructor refuses
/// `rdf:langString`, making `Any(rdf:langString)` unrepresentable via safe
/// API.
///
/// [s]: https://www.w3.org/TR/rdf11-concepts/#section-Graph-Literal
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LiteralType {
    /// Typed literal with a non-`rdf:langString` datatype.
    Any(Datatype),

    /// Language-tagged literal.
    LangString(LangTagBuf),
}

impl LiteralType {
    /// Constructs a typed literal type. Returns
    /// [`RdfLangStringDatatypeError`] if `iri == rdf:langString`; use
    /// [`LiteralType::lang_string`] for language-tagged literals.
    pub fn typed(iri: IriBuf) -> Result<Self, RdfLangStringDatatypeError> {
        Datatype::new(iri).map(Self::Any)
    }

    /// Constructs a language-tagged literal type.
    pub const fn lang_string(tag: LangTagBuf) -> Self {
        Self::LangString(tag)
    }

    /// Returns the `xsd:string` literal type.
    pub fn xsd_string() -> Self {
        Self::Any(Datatype::xsd_string())
    }

    pub const fn is_lang_string(&self) -> bool {
        matches!(self, Self::LangString(_))
    }

    pub fn lang_tag(&self) -> Option<&LangTag> {
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

    pub fn as_ref(&self) -> LiteralTypeRef<'_> {
        match self {
            Self::Any(d) => LiteralTypeRef::Any(d.as_iri()),
            Self::LangString(l) => LiteralTypeRef::LangString(l),
        }
    }

    pub fn as_cow(&self) -> CowLiteralType<'_> {
        match self {
            Self::Any(d) => CowLiteralType::Any(Cow::Borrowed(d)),
            Self::LangString(l) => CowLiteralType::LangString(Cow::Borrowed(l)),
        }
    }

    pub fn into_cow(self) -> CowLiteralType<'static> {
        match self {
            Self::Any(d) => CowLiteralType::Any(Cow::Owned(d)),
            Self::LangString(l) => CowLiteralType::LangString(Cow::Owned(l)),
        }
    }
}

impl TryFrom<IriBuf> for LiteralType {
    type Error = RdfLangStringDatatypeError;
    fn try_from(value: IriBuf) -> Result<Self, Self::Error> {
        Self::typed(value)
    }
}

impl From<Datatype> for LiteralType {
    fn from(value: Datatype) -> Self {
        Self::Any(value)
    }
}

impl From<LangTagBuf> for LiteralType {
    fn from(value: LangTagBuf) -> Self {
        Self::LangString(value)
    }
}

impl RdfDisplay for LiteralType {
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
