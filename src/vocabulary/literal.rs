use super::{EmbedIntoVocabulary, EmbeddedIntoVocabulary};
use crate::{Literal, LiteralRef};

/// Literal value vocabulary.
pub trait LiteralVocabulary {
    /// Literal identifier type.
    type Literal;

    fn literal<'l>(&'l self, id: &'l Self::Literal) -> Option<LiteralRef<'l>>;

    fn owned_literal(&self, id: Self::Literal) -> Result<Literal, Self::Literal>;

    /// Returns the vocabulary id of the given literal identifier, if any.
    fn get_literal(&self, id: LiteralRef<'_>) -> Option<Self::Literal>;
}

impl<V: LiteralVocabulary> LiteralVocabulary for &V {
    type Literal = V::Literal;

    fn literal<'l>(&'l self, id: &'l Self::Literal) -> Option<LiteralRef<'l>> {
        V::literal(*self, id)
    }

    fn owned_literal(&self, id: Self::Literal) -> Result<Literal, Self::Literal> {
        V::owned_literal(*self, id)
    }

    fn get_literal(&self, id: LiteralRef<'_>) -> Option<Self::Literal> {
        V::get_literal(*self, id)
    }
}

impl<V: LiteralVocabulary> LiteralVocabulary for &mut V {
    type Literal = V::Literal;

    fn literal<'l>(&'l self, id: &'l Self::Literal) -> Option<LiteralRef<'l>> {
        V::literal(*self, id)
    }

    fn owned_literal(&self, id: Self::Literal) -> Result<Literal, Self::Literal> {
        V::owned_literal(*self, id)
    }

    fn get_literal(&self, id: LiteralRef<'_>) -> Option<Self::Literal> {
        V::get_literal(*self, id)
    }
}

/// Mutable literal value vocabulary.
pub trait LiteralVocabularyMut: LiteralVocabulary {
    fn insert_literal(&mut self, value: LiteralRef<'_>) -> Self::Literal;

    fn insert_owned_literal(&mut self, value: Literal) -> Self::Literal {
        self.insert_literal(value.as_ref())
    }
}

impl<V: LiteralVocabularyMut> LiteralVocabularyMut for &mut V {
    fn insert_literal(&mut self, value: LiteralRef<'_>) -> Self::Literal {
        V::insert_literal(*self, value)
    }

    fn insert_owned_literal(&mut self, value: Literal) -> Self::Literal {
        V::insert_owned_literal(*self, value)
    }
}

impl<'a, V: LiteralVocabularyMut> EmbedIntoVocabulary<V> for LiteralRef<'a> {
    type Embedded = V::Literal;

    fn embed_into_vocabulary(self, vocabulary: &mut V) -> Self::Embedded {
        vocabulary.insert_literal(self)
    }
}

impl<V: LiteralVocabularyMut> EmbedIntoVocabulary<V> for Literal {
    type Embedded = V::Literal;

    fn embed_into_vocabulary(self, vocabulary: &mut V) -> Self::Embedded {
        vocabulary.insert_owned_literal(self)
    }
}

impl<'a, V: LiteralVocabularyMut> EmbeddedIntoVocabulary<V> for LiteralRef<'a> {
    type Embedded = V::Literal;

    fn embedded_into_vocabulary(&self, vocabulary: &mut V) -> Self::Embedded {
        vocabulary.insert_literal(*self)
    }
}

impl<V: LiteralVocabularyMut> EmbeddedIntoVocabulary<V> for Literal {
    type Embedded = V::Literal;

    fn embedded_into_vocabulary(&self, vocabulary: &mut V) -> Self::Embedded {
        vocabulary.insert_literal(self.as_ref())
    }
}
