//! RDF 1.2 base direction for directional language strings.
//!
//! Per [RDF 1.2 Concepts §3.4][s], a directional language-tagged string
//! pairs a language tag with an explicit base direction, either left-to-right
//! (`ltr`) or right-to-left (`rtl`).
//!
//! [s]: https://www.w3.org/TR/rdf12-concepts/#dfn-dir-lang-string

use core::fmt;

use crate::RdfDisplay;

/// Returned by [`Direction::parse`] for an unknown direction string.
#[derive(Debug, thiserror::Error)]
#[error("invalid base direction: {0:?} (expected \"ltr\" or \"rtl\")")]
pub struct InvalidDirection(pub String);

/// Base direction of a directional language-tagged string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "lowercase"))]
pub enum Direction {
    /// Left-to-right.
    Ltr,
    /// Right-to-left.
    Rtl,
}

impl Direction {
    /// Lexical form, as used in N-Triples / N-Quads after `--`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }

    /// `const` parser, suitable for use inside macros.
    pub const fn parse_const(s: &str) -> Option<Self> {
        match s.as_bytes() {
            b"ltr" => Some(Self::Ltr),
            b"rtl" => Some(Self::Rtl),
            _ => None,
        }
    }

    /// Parses a direction string.
    pub fn parse(s: &str) -> Result<Self, InvalidDirection> {
        Self::parse_const(s).ok_or_else(|| InvalidDirection(s.to_owned()))
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl RdfDisplay for Direction {
    fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for Direction {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl core::str::FromStr for Direction {
    type Err = InvalidDirection;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
