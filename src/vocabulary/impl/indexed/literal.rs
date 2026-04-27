use crate::vocabulary::{ExtractFromVocabulary, ExtractedFromVocabulary, LiteralVocabulary};
use crate::{Literal, LiteralRef};

/// Literal index.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct LiteralIndex(usize);

impl From<usize> for LiteralIndex {
    fn from(i: usize) -> Self {
        Self(i)
    }
}

impl From<LiteralIndex> for usize {
    fn from(value: LiteralIndex) -> Self {
        value.0
    }
}

impl IndexedLiteral for LiteralIndex {
    fn literal_index(&self) -> LiteralOrIndex<&'_ Literal> {
        LiteralOrIndex::Index(self.0)
    }

    fn into_literal_index(self) -> LiteralOrIndex<Literal> {
        LiteralOrIndex::Index(self.0)
    }
}

impl<V> ExtractFromVocabulary<V> for LiteralIndex
where
    V: LiteralVocabulary<Literal = LiteralIndex>,
{
    type Extracted = Literal;

    fn extract_from_vocabulary(self, vocabulary: &V) -> Self::Extracted {
        vocabulary.owned_literal(self).unwrap()
    }
}

impl<V> ExtractedFromVocabulary<V> for LiteralIndex
where
    V: LiteralVocabulary<Literal = LiteralIndex>,
{
    type Extracted = Literal;

    fn extracted_from_vocabulary(&self, vocabulary: &V) -> Self::Extracted {
        vocabulary.literal(self).unwrap().to_owned()
    }
}

impl TryFrom<Literal> for LiteralIndex {
    type Error = Literal;

    fn try_from(value: Literal) -> Result<Self, Self::Error> {
        Err(value)
    }
}

impl<'a> TryFrom<LiteralRef<'a>> for LiteralIndex {
    type Error = ();

    fn try_from(_value: LiteralRef<'a>) -> Result<Self, Self::Error> {
        Err(())
    }
}

#[cfg(feature = "contextual")]
impl<V: crate::vocabulary::LiteralVocabulary<Literal = Self>> contextual::DisplayWithContext<V>
    for LiteralIndex
{
    fn fmt_with(&self, vocabulary: &V, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&vocabulary.literal(self).unwrap(), f)
    }
}

#[cfg(feature = "contextual")]
impl<V: crate::vocabulary::LiteralVocabulary<Literal = Self>> crate::RdfDisplayWithContext<V>
    for LiteralIndex
{
    fn rdf_fmt_with(&self, vocabulary: &V, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        crate::RdfDisplay::rdf_fmt(&vocabulary.literal(self).unwrap(), f)
    }
}

/// Literal or index.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum LiteralOrIndex<I> {
    /// Index in the vocabulary.
    Index(usize),

    /// Non indexed literal.
    Literal(I),
}

impl<I> From<usize> for LiteralOrIndex<I> {
    fn from(i: usize) -> Self {
        Self::Index(i)
    }
}

impl<L: TryFrom<Literal>> TryFrom<Literal> for LiteralOrIndex<L> {
    type Error = L::Error;

    fn try_from(value: Literal) -> Result<Self, Self::Error> {
        Ok(Self::Literal(L::try_from(value)?))
    }
}

impl<'a, L: TryFrom<LiteralRef<'a>>> TryFrom<LiteralRef<'a>> for LiteralOrIndex<L> {
    type Error = L::Error;

    fn try_from(literal: LiteralRef<'a>) -> Result<Self, Self::Error> {
        Ok(Self::Literal(L::try_from(literal)?))
    }
}

/// Partly indexed literal value type.
pub trait IndexedLiteral:
    From<usize> + for<'a> TryFrom<LiteralRef<'a>> + TryFrom<Literal, Error = Literal>
{
    fn literal_index(&self) -> LiteralOrIndex<&Literal>;

    fn into_literal_index(self) -> LiteralOrIndex<Literal>;
}

impl<L> IndexedLiteral for LiteralOrIndex<L>
where
    L: AsRef<Literal>
        + Into<Literal>
        + for<'a> TryFrom<LiteralRef<'a>>
        + TryFrom<Literal, Error = Literal>,
{
    fn literal_index(&self) -> LiteralOrIndex<&Literal> {
        match self {
            Self::Literal(i) => LiteralOrIndex::Literal(i.as_ref()),
            Self::Index(i) => LiteralOrIndex::Index(*i),
        }
    }

    fn into_literal_index(self) -> LiteralOrIndex<Literal> {
        match self {
            Self::Literal(i) => LiteralOrIndex::Literal(i.into()),
            Self::Index(i) => LiteralOrIndex::Index(i),
        }
    }
}
