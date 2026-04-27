use crate::{
    Interpretation, Literal, LiteralRef,
    vocabulary::{LiteralVocabulary, LiteralVocabularyMut},
};

/// Literal value interpretation.
pub trait LiteralInterpretation<L>: Interpretation {
    /// Returns the interpretation of the given literal value, if any.
    fn literal_interpretation(&self, literal: &L) -> Option<Self::Resource>;

    fn lexical_literal_interpretation<V: LiteralVocabulary<Literal = L>>(
        &self,
        vocabulary: &V,
        literal: LiteralRef<'_>,
    ) -> Option<Self::Resource> {
        vocabulary
            .get_literal(literal)
            .and_then(|l| self.literal_interpretation(&l))
    }
}

impl<L, T: LiteralInterpretation<L>> LiteralInterpretation<L> for &T {
    fn literal_interpretation(&self, literal: &L) -> Option<Self::Resource> {
        T::literal_interpretation(*self, literal)
    }

    fn lexical_literal_interpretation<V: LiteralVocabulary<Literal = L>>(
        &self,
        vocabulary: &V,
        literal: LiteralRef<'_>,
    ) -> Option<Self::Resource> {
        T::lexical_literal_interpretation(*self, vocabulary, literal)
    }
}

impl<L, T: LiteralInterpretation<L>> LiteralInterpretation<L> for &mut T {
    fn literal_interpretation(&self, literal: &L) -> Option<Self::Resource> {
        T::literal_interpretation(*self, literal)
    }

    fn lexical_literal_interpretation<V: LiteralVocabulary<Literal = L>>(
        &self,
        vocabulary: &V,
        literal: LiteralRef<'_>,
    ) -> Option<Self::Resource> {
        T::lexical_literal_interpretation(*self, vocabulary, literal)
    }
}

/// Mutable literal value interpretation.
pub trait LiteralInterpretationMut<L = Literal>: Interpretation {
    /// Interprets the given literal value.
    fn interpret_literal(&mut self, literal: L) -> Self::Resource;

    fn interpret_lexical_literal<V: LiteralVocabularyMut<Literal = L>>(
        &mut self,
        vocabulary: &mut V,
        literal: LiteralRef<'_>,
    ) -> Self::Resource {
        self.interpret_literal(vocabulary.insert_literal(literal))
    }

    fn interpret_owned_lexical_literal<V: LiteralVocabularyMut<Literal = L>>(
        &mut self,
        vocabulary: &mut V,
        literal: Literal,
    ) -> Self::Resource {
        self.interpret_literal(vocabulary.insert_owned_literal(literal))
    }
}

pub trait ReverseLiteralInterpretation: Interpretation {
    type Literal;

    type Literals<'a>: Clone + Iterator<Item = &'a Self::Literal>
    where
        Self: 'a;

    fn literals_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Literals<'a>;
}

impl<T: ReverseLiteralInterpretation> ReverseLiteralInterpretation for &T {
    type Literal = T::Literal;
    type Literals<'a>
        = T::Literals<'a>
    where
        Self: 'a;

    fn literals_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Literals<'a> {
        T::literals_of(*self, id)
    }
}

impl<T: ReverseLiteralInterpretation> ReverseLiteralInterpretation for &mut T {
    type Literal = T::Literal;
    type Literals<'a>
        = T::Literals<'a>
    where
        Self: 'a;

    fn literals_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Literals<'a> {
        T::literals_of(*self, id)
    }
}
