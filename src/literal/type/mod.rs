use core::fmt;
use iri_rs::{Iri, IriBuf};
use langtag::{LangTag, LangTagBuf};
use std::{borrow::Cow, fmt::Write, ops::Deref};

use super::{Datatype, Direction, ReservedDatatypeError};
use crate::{RdfDisplay, XSD_STRING};

mod r#ref;
pub use r#ref::*;

mod cow;
pub use cow::*;

/// RDF literal type.
///
/// Per [RDF 1.2 Concepts §3.4][s], a literal is either:
/// - a typed literal with a datatype IRI other than `rdf:langString` /
///   `rdf:dirLangString` ([`LiteralType::Any`]), or
/// - a language-tagged literal with no base direction
///   ([`LiteralType::LangString`]), or
/// - a directional language-tagged literal with an explicit base direction
///   ([`LiteralType::DirLangString`]).
///
/// `LiteralType::Any` wraps a [`Datatype`] whose constructor refuses both
/// `rdf:langString` and `rdf:dirLangString`, making
/// `Any(rdf:langString)` / `Any(rdf:dirLangString)` unrepresentable via the
/// safe API.
///
/// [s]: https://www.w3.org/TR/rdf12-concepts/#section-Graph-Literal
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LiteralType {
    /// Typed literal with a non-`rdf:langString`, non-`rdf:dirLangString`
    /// datatype.
    Any(Datatype),

    /// Language-tagged literal with no base direction.
    LangString(LangTagBuf),

    /// Directional language-tagged literal (RDF 1.2).
    DirLangString {
        /// Language tag.
        tag: LangTagBuf,
        /// Base direction.
        direction: Direction,
    },
}

impl LiteralType {
    /// Constructs a typed literal type. Returns
    /// [`ReservedDatatypeError`] if `iri` is `rdf:langString` or
    /// `rdf:dirLangString`; use [`LiteralType::lang_string`] or
    /// [`LiteralType::dir_lang_string`] for those cases.
    pub fn typed(iri: IriBuf) -> Result<Self, ReservedDatatypeError> {
        Datatype::new(iri).map(Self::Any)
    }

    /// Constructs a language-tagged literal type with no base direction.
    pub const fn lang_string(tag: LangTagBuf) -> Self {
        Self::LangString(tag)
    }

    /// Constructs a directional language-tagged literal type (RDF 1.2).
    pub const fn dir_lang_string(tag: LangTagBuf, direction: Direction) -> Self {
        Self::DirLangString { tag, direction }
    }

    /// Returns the `xsd:string` literal type.
    pub fn xsd_string() -> Self {
        Self::Any(Datatype::xsd_string())
    }

    /// Returns `true` for both [`Self::LangString`] and [`Self::DirLangString`]
    /// — i.e. any language-tagged string.
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

    pub fn lang_tag(&self) -> Option<&LangTag> {
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

    pub fn as_ref(&self) -> LiteralTypeRef<'_> {
        match self {
            Self::Any(d) => LiteralTypeRef::Any(d.as_datatype_ref()),
            Self::LangString(l) => LiteralTypeRef::LangString(l),
            Self::DirLangString { tag, direction } => LiteralTypeRef::DirLangString {
                tag,
                direction: *direction,
            },
        }
    }

    pub fn as_cow(&self) -> CowLiteralType<'_> {
        match self {
            Self::Any(d) => CowLiteralType::Any(Cow::Borrowed(d)),
            Self::LangString(l) => CowLiteralType::LangString(Cow::Borrowed(l)),
            Self::DirLangString { tag, direction } => CowLiteralType::DirLangString {
                tag: Cow::Borrowed(tag),
                direction: *direction,
            },
        }
    }

    pub fn into_cow(self) -> CowLiteralType<'static> {
        match self {
            Self::Any(d) => CowLiteralType::Any(Cow::Owned(d)),
            Self::LangString(l) => CowLiteralType::LangString(Cow::Owned(l)),
            Self::DirLangString { tag, direction } => CowLiteralType::DirLangString {
                tag: Cow::Owned(tag),
                direction,
            },
        }
    }
}

impl TryFrom<IriBuf> for LiteralType {
    type Error = ReservedDatatypeError;
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
            Self::DirLangString { tag, direction } => {
                f.write_char('@')?;
                tag.rdf_fmt(f)?;
                f.write_str("--")?;
                f.write_str(direction.as_str())
            }
        }
    }
}
