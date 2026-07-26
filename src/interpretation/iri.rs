use iri_rs::{Iri, IriBuf};

use crate::vocabulary::{IriVocabulary, IriVocabularyMut};

use super::Interpretation;

/// IRI Interpretation.
pub trait IriInterpretation<I: ?Sized>: Interpretation {
    /// Returns the interpretation of the given IRI, if any.
    fn iri_interpretation(&self, iri: &I) -> Option<Self::Resource>;

    fn lexical_iri_interpretation(&self, vocabulary: &impl IriVocabulary<Iri = I>, iri: Iri<&str>) -> Option<Self::Resource>
    where
        I: Sized,
    {
        vocabulary.get(iri).and_then(|iri| self.iri_interpretation(&iri))
    }
}

impl<I, T: IriInterpretation<I>> IriInterpretation<I> for &T {
    fn iri_interpretation(&self, iri: &I) -> Option<Self::Resource> {
        T::iri_interpretation(*self, iri)
    }
}

impl<I, T: IriInterpretation<I>> IriInterpretation<I> for &mut T {
    fn iri_interpretation(&self, iri: &I) -> Option<Self::Resource> {
        T::iri_interpretation(*self, iri)
    }
}

/// Mutable IRI interpretation.
pub trait IriInterpretationMut<I = IriBuf>: Interpretation {
    /// Interprets the given IRI.
    fn interpret_iri(&mut self, iri: I) -> Self::Resource;

    fn interpret_lexical_iri(&mut self, vocabulary: &mut impl IriVocabularyMut<Iri = I>, iri: Iri<&str>) -> Self::Resource {
        self.interpret_iri(vocabulary.insert(iri))
    }

    fn interpret_owned_lexical_iri(&mut self, vocabulary: &mut impl IriVocabularyMut<Iri = I>, iri: IriBuf) -> Self::Resource {
        self.interpret_iri(vocabulary.insert_owned(iri))
    }
}

impl<I, T: IriInterpretationMut<I>> IriInterpretationMut<I> for &mut T {
    fn interpret_iri(&mut self, iri: I) -> Self::Resource {
        T::interpret_iri(*self, iri)
    }

    fn interpret_lexical_iri(&mut self, vocabulary: &mut impl IriVocabularyMut<Iri = I>, iri: Iri<&str>) -> Self::Resource {
        T::interpret_lexical_iri(*self, vocabulary, iri)
    }

    fn interpret_owned_lexical_iri(&mut self, vocabulary: &mut impl IriVocabularyMut<Iri = I>, iri: IriBuf) -> Self::Resource {
        T::interpret_owned_lexical_iri(*self, vocabulary, iri)
    }
}

pub trait ReverseIriInterpretation: Interpretation {
    type Iri;
    type Iris<'a>: Clone + Iterator<Item = &'a Self::Iri>
    where
        Self: 'a;

    fn iris_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Iris<'a>;
}

impl<T: ReverseIriInterpretation> ReverseIriInterpretation for &T {
    type Iri = T::Iri;
    type Iris<'a>
        = T::Iris<'a>
    where
        Self: 'a;

    fn iris_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Iris<'a> {
        T::iris_of(*self, id)
    }
}

impl<T: ReverseIriInterpretation> ReverseIriInterpretation for &mut T {
    type Iri = T::Iri;
    type Iris<'a>
        = T::Iris<'a>
    where
        Self: 'a;

    fn iris_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Iris<'a> {
        T::iris_of(*self, id)
    }
}

pub trait ReverseIriInterpretationMut: ReverseIriInterpretation {
    fn assign_iri(&mut self, id: &Self::Resource, iri: Self::Iri) -> bool;
}

impl<T: ReverseIriInterpretationMut> ReverseIriInterpretationMut for &mut T {
    fn assign_iri(&mut self, id: &Self::Resource, iri: Self::Iri) -> bool {
        T::assign_iri(*self, id, iri)
    }
}
