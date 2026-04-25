use std::{borrow::Cow, fmt, ops::Deref};

use iri_rs::{Iri, IriBuf};

use crate::{RDF_LANG_STRING, RdfDisplay, XSD_STRING};

/// Error returned when attempting to construct a [`Datatype`] from
/// `rdf:langString`.
///
/// Per [RDF 1.1 Concepts §3.3][s], `rdf:langString` is reserved for
/// language-tagged literals and cannot appear as a plain typed-literal
/// datatype. Use [`LiteralType::lang_string`](crate::LiteralType::lang_string)
/// for language-tagged literals instead.
///
/// [s]: https://www.w3.org/TR/rdf11-concepts/#dfn-language-tagged-string
#[derive(Debug, thiserror::Error)]
#[error("rdf:langString cannot be a typed-literal datatype; use LiteralType::lang_string")]
pub struct RdfLangStringDatatypeError(pub IriBuf);

/// Validated typed-literal datatype IRI.
///
/// Per [RDF 1.1 Concepts §3.3][s], the wrapped IRI is guaranteed to differ
/// from `rdf:langString`. Construct via [`Datatype::new`].
///
/// [s]: https://www.w3.org/TR/rdf11-concepts/#section-Graph-Literal
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct Datatype(IriBuf);

impl Datatype {
    /// Creates a `Datatype` from `iri`, returning [`RdfLangStringDatatypeError`]
    /// if `iri == rdf:langString`.
    pub fn new(iri: IriBuf) -> Result<Self, RdfLangStringDatatypeError> {
        if iri.as_str() == RDF_LANG_STRING.as_str() {
            Err(RdfLangStringDatatypeError(iri))
        } else {
            Ok(Self(iri))
        }
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

    /// Constructs a `Datatype` without validating that `iri != rdf:langString`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `iri.as_str() != RDF_LANG_STRING.as_str()`.
    /// Violating this lets `LiteralType::Any(rdf:langString)` exist via safe
    /// API and breaks the RDF 1.1 typed-literal / lang-tagged distinction.
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
    type Error = RdfLangStringDatatypeError;
    fn try_from(value: IriBuf) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Borrowed counterpart of [`Datatype`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct DatatypeRef<'a>(Iri<&'a str>);

impl<'a> DatatypeRef<'a> {
    /// Creates a `DatatypeRef` from `iri`, returning [`RdfLangStringDatatypeError`]
    /// if `iri == rdf:langString`.
    pub fn new(iri: Iri<&'a str>) -> Result<Self, RdfLangStringDatatypeError> {
        if iri.as_str() == RDF_LANG_STRING.as_str() {
            Err(RdfLangStringDatatypeError(iri.into()))
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

    /// Constructs a `DatatypeRef` without validating that `iri != rdf:langString`.
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
        Datatype::new(iri).map_err(|RdfLangStringDatatypeError(bad)| {
            <D::Error as serde::de::Error>::invalid_value(
                serde::de::Unexpected::Str(bad.as_str()),
                &"a non-rdf:langString datatype IRI",
            )
        })
    }
}
