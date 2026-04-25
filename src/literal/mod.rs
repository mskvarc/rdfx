use crate::RdfDisplay;
use langtag::{LangTag, LangTagBuf};
use std::{borrow::Borrow, fmt};

mod datatype;
pub use datatype::*;

mod r#ref;
pub use r#ref::*;

mod cow;
pub use cow::*;

mod r#type;
pub use r#type::*;

/// RDF Literal.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Literal {
    /// Literal value.
    pub value: String,

    /// Literal type.
    pub type_: LiteralType,
}

impl Literal {
    pub fn new(value: impl Into<String>, type_: impl Into<LiteralType>) -> Self {
        Self {
            value: value.into(),
            type_: type_.into(),
        }
    }

    /// Constructs a simple literal — equivalent to a typed literal with
    /// datatype `xsd:string` per [RDF 1.1 Concepts §3.3][s].
    ///
    /// Round-trips: `Literal::plain(v) == Literal::typed(v, Datatype::xsd_string())`.
    ///
    /// [s]: https://www.w3.org/TR/rdf11-concepts/#section-Graph-Literal
    pub fn plain(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            type_: LiteralType::xsd_string(),
        }
    }

    /// Constructs a typed literal with the given datatype.
    pub fn typed(value: impl Into<String>, dt: Datatype) -> Self {
        Self {
            value: value.into(),
            type_: LiteralType::Any(dt),
        }
    }

    /// Constructs a language-tagged literal.
    pub fn lang_string(value: impl Into<String>, tag: LangTagBuf) -> Self {
        Self {
            value: value.into(),
            type_: LiteralType::LangString(tag),
        }
    }

    pub const fn as_type(&self) -> &LiteralType {
        &self.type_
    }

    pub const fn as_type_mut(&mut self) -> &mut LiteralType {
        &mut self.type_
    }

    pub fn into_type(self) -> LiteralType {
        self.type_
    }

    pub fn into_value(self) -> String {
        self.value
    }

    pub fn into_parts(self) -> (String, LiteralType) {
        (self.value, self.type_)
    }

    pub fn as_str(&self) -> &str {
        self.value.as_ref()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.value.as_ref()
    }

    pub const fn is_lang_string(&self) -> bool {
        self.type_.is_lang_string()
    }

    pub fn lang_tag(&self) -> Option<&LangTag> {
        self.type_.lang_tag()
    }

    pub fn as_ref(&self) -> LiteralRef<'_> {
        LiteralRef::new(&self.value, self.type_.as_ref())
    }

    pub fn as_cow(&self) -> CowLiteral<'_> {
        CowLiteral::new(&self.value, self.type_.as_cow())
    }

    pub fn into_cow(self) -> CowLiteral<'static> {
        CowLiteral::new(self.value, self.type_.into_cow())
    }
}

impl Borrow<str> for Literal {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Literal {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.rdf_fmt(f)?;
        if self.type_.is_xsd_string() { Ok(()) } else { self.type_.rdf_fmt(f) }
    }
}

impl RdfDisplay for Literal {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.rdf_fmt(f)?;
        if self.type_.is_xsd_string() { Ok(()) } else { self.type_.rdf_fmt(f) }
    }
}
