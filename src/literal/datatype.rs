use std::{borrow::Cow, fmt, ops::Deref};

use iri_rs::{Iri, IriBuf};

use crate::{RDF_DIR_LANG_STRING, RDF_LANG_STRING, RdfDisplay, XSD_STRING};

/// Error returned when attempting to construct a [`Datatype`] from one of the
/// RDF-reserved language-string datatype IRIs.
///
/// Per [RDF 1.2 Concepts §3.4][s], `rdf:langString` and `rdf:dirLangString`
/// are reserved for language-tagged literals (with and without an explicit
/// base direction) and cannot appear as a plain typed-literal datatype. Use
/// [`LiteralType::lang_string`](crate::LiteralType::lang_string) or
/// [`LiteralType::dir_lang_string`](crate::LiteralType::dir_lang_string)
/// instead.
///
/// [s]: https://www.w3.org/TR/rdf12-concepts/#section-Graph-Literal
#[derive(Debug, thiserror::Error)]
pub enum ReservedDatatypeError {
    /// `rdf:langString` was used as a typed-literal datatype.
    #[error("rdf:langString cannot be a typed-literal datatype; use LiteralType::lang_string")]
    LangString(IriBuf),
    /// `rdf:dirLangString` was used as a typed-literal datatype.
    #[error("rdf:dirLangString cannot be a typed-literal datatype; use LiteralType::dir_lang_string")]
    DirLangString(IriBuf),
}

impl ReservedDatatypeError {
    /// Returns the IRI that triggered the error.
    pub fn iri(&self) -> &IriBuf {
        match self {
            Self::LangString(i) | Self::DirLangString(i) => i,
        }
    }

    /// Consumes the error, returning the offending IRI.
    pub fn into_iri(self) -> IriBuf {
        match self {
            Self::LangString(i) | Self::DirLangString(i) => i,
        }
    }
}

fn classify_reserved(iri: IriBuf) -> Result<IriBuf, ReservedDatatypeError> {
    if iri.as_str() == RDF_LANG_STRING.as_str() {
        Err(ReservedDatatypeError::LangString(iri))
    } else if iri.as_str() == RDF_DIR_LANG_STRING.as_str() {
        Err(ReservedDatatypeError::DirLangString(iri))
    } else {
        Ok(iri)
    }
}

/// Validated typed-literal datatype IRI.
///
/// Per [RDF 1.2 Concepts §3.4][s], the wrapped IRI is guaranteed to differ
/// from `rdf:langString` and `rdf:dirLangString`. Construct via
/// [`Datatype::new`].
///
/// [s]: https://www.w3.org/TR/rdf12-concepts/#section-Graph-Literal
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct Datatype(IriBuf);

impl Datatype {
    /// Creates a `Datatype` from `iri`, returning [`ReservedDatatypeError`]
    /// if `iri` is `rdf:langString` or `rdf:dirLangString`.
    pub fn new(iri: IriBuf) -> Result<Self, ReservedDatatypeError> {
        classify_reserved(iri).map(Self)
    }

    /// Returns the `xsd:string` datatype.
    pub fn xsd_string() -> Self {
        Self(IriBuf::from(XSD_STRING))
    }

    /// Consumes the `Datatype`, returning the underlying IRI.
    pub fn into_iri(self) -> IriBuf {
        self.0
    }

    /// Borrows the underlying IRI.
    pub fn as_iri(&self) -> Iri<&str> {
        self.0.as_ref()
    }

    /// Borrows as a [`DatatypeRef`].
    pub fn as_datatype_ref(&self) -> DatatypeRef<'_> {
        DatatypeRef(self.0.as_ref())
    }

    /// Constructs a `Datatype` without validating that the IRI is not
    /// `rdf:langString` or `rdf:dirLangString`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the IRI differs from both reserved
    /// language-string datatype IRIs. Violating this lets
    /// `LiteralType::Any(rdf:langString)` (or `Any(rdf:dirLangString)`) exist
    /// via safe API and breaks the typed-literal / lang-tagged distinction.
    pub const unsafe fn new_unchecked(iri: IriBuf) -> Self {
        Self(iri)
    }
}

impl Deref for Datatype {
    type Target = IriBuf;
    fn deref(&self) -> &IriBuf {
        &self.0
    }
}

impl AsRef<IriBuf> for Datatype {
    fn as_ref(&self) -> &IriBuf {
        &self.0
    }
}

impl fmt::Display for Datatype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl RdfDisplay for Datatype {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.rdf_fmt(f)
    }
}

impl TryFrom<IriBuf> for Datatype {
    type Error = ReservedDatatypeError;
    fn try_from(value: IriBuf) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Borrowed counterpart of [`Datatype`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct DatatypeRef<'a>(Iri<&'a str>);

impl<'a> DatatypeRef<'a> {
    /// Creates a `DatatypeRef` from `iri`, returning [`ReservedDatatypeError`]
    /// if `iri` is `rdf:langString` or `rdf:dirLangString`.
    pub fn new(iri: Iri<&'a str>) -> Result<Self, ReservedDatatypeError> {
        if iri.as_str() == RDF_LANG_STRING.as_str() {
            Err(ReservedDatatypeError::LangString(iri.into()))
        } else if iri.as_str() == RDF_DIR_LANG_STRING.as_str() {
            Err(ReservedDatatypeError::DirLangString(iri.into()))
        } else {
            Ok(Self(iri))
        }
    }

    /// Returns the underlying IRI reference.
    pub const fn as_iri(self) -> Iri<&'a str> {
        self.0
    }

    /// Promotes this reference into an owned [`Datatype`].
    pub fn to_owned(self) -> Datatype {
        // Safety: invariant established at construction.
        unsafe { Datatype::new_unchecked(self.0.into()) }
    }

    /// Constructs a `DatatypeRef` without validating that the IRI is not
    /// `rdf:langString` or `rdf:dirLangString`.
    ///
    /// # Safety
    ///
    /// Same contract as [`Datatype::new_unchecked`].
    pub const unsafe fn new_unchecked(iri: Iri<&'a str>) -> Self {
        Self(iri)
    }
}

impl fmt::Display for DatatypeRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl RdfDisplay for DatatypeRef<'_> {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.rdf_fmt(f)
    }
}

/// Owned-or-borrowed [`Datatype`].
pub type CowDatatype<'a> = Cow<'a, Datatype>;

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Datatype {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let iri = IriBuf::deserialize(deserializer)?;
        Datatype::new(iri).map_err(|e| {
            <D::Error as serde::de::Error>::invalid_value(
                serde::de::Unexpected::Str(e.iri().as_str()),
                &"a non-rdf:langString, non-rdf:dirLangString datatype IRI",
            )
        })
    }
}
