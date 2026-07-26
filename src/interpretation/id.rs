use crate::{
    Id,
    IdRef,
    vocabulary::{BlankIdVocabulary, BlankIdVocabularyMut, IriVocabulary, IriVocabularyMut},
};

use super::{
    BlankIdInterpretation,
    BlankIdInterpretationMut,
    IriInterpretation,
    IriInterpretationMut,
    ReverseBlankIdInterpretation,
    ReverseBlankIdInterpretationMut,
    ReverseIriInterpretation,
    ReverseIriInterpretationMut,
};

/// Node identifier interpretation.
pub trait IdInterpretation<I, B>: IriInterpretation<I> + BlankIdInterpretation<B> {
    fn lexical_id_interpretation(&self, vocabulary: &(impl IriVocabulary<Iri = I> + BlankIdVocabulary<BlankId = B>), id: IdRef<'_>) -> Option<Self::Resource> {
        match id {
            IdRef::Iri(i) => self.lexical_iri_interpretation(vocabulary, i),
            IdRef::BlankId(b) => self.lexical_blank_id_interpretation(vocabulary, b),
        }
    }
}

impl<I, B, T: IriInterpretation<I> + BlankIdInterpretation<B>> IdInterpretation<I, B> for T {}

/// Node identifier interpretation.
pub trait IdInterpretationMut<I, B>: IriInterpretationMut<I> + BlankIdInterpretationMut<B> {
    fn interpret_lexical_id(&mut self, vocabulary: &mut (impl IriVocabularyMut<Iri = I> + BlankIdVocabularyMut<BlankId = B>), id: IdRef<'_>) -> Self::Resource {
        match id {
            IdRef::Iri(i) => self.interpret_lexical_iri(vocabulary, i),
            IdRef::BlankId(b) => self.interpret_lexical_blank_id(vocabulary, b),
        }
    }

    fn interpret_owned_lexical_id(&mut self, vocabulary: &mut (impl IriVocabularyMut<Iri = I> + BlankIdVocabularyMut<BlankId = B>), id: Id) -> Self::Resource {
        match id {
            Id::Iri(i) => self.interpret_owned_lexical_iri(vocabulary, i),
            Id::BlankId(b) => self.interpret_owned_lexical_blank_id(vocabulary, b),
        }
    }
}

impl<I, B, T: IriInterpretationMut<I> + BlankIdInterpretationMut<B>> IdInterpretationMut<I, B> for T {}

/// Reverse node identifier interpretation.
///
/// Used to retrieve the node identifiers of a given resource.
pub trait ReverseIdInterpretation: ReverseIriInterpretation + ReverseBlankIdInterpretation {
    fn id_iris_of<'a>(&'a self, id: &'a Self::Resource) -> IdsOf<'a, Self> {
        IdsOf {
            iris: ReverseIriInterpretation::iris_of(self, id),
            blanks: ReverseBlankIdInterpretation::blank_ids_of(self, id),
        }
    }
}

impl<I: ?Sized + ReverseIriInterpretation + ReverseBlankIdInterpretation> ReverseIdInterpretation for I {}

pub struct IdsOf<'a, I: 'a + ?Sized + ReverseIdInterpretation> {
    iris: <I as ReverseIriInterpretation>::Iris<'a>,
    blanks: <I as ReverseBlankIdInterpretation>::BlankIds<'a>,
}

impl<'a, I: 'a + ?Sized + ReverseIdInterpretation> Clone for IdsOf<'a, I> {
    fn clone(&self) -> Self {
        Self {
            iris: self.iris.clone(),
            blanks: self.blanks.clone(),
        }
    }
}

impl<'a, I: 'a + ?Sized + ReverseIdInterpretation> Copy for IdsOf<'a, I>
where
    <I as ReverseIriInterpretation>::Iris<'a>: Copy,
    <I as ReverseBlankIdInterpretation>::BlankIds<'a>: Copy,
{
}

/// Reference-form id holding refs into vocabulary stores.
pub enum BorrowedId<'a, I, B> {
    Iri(&'a I),
    BlankId(&'a B),
}

impl<'a, I, B> Clone for BorrowedId<'a, I, B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, I, B> Copy for BorrowedId<'a, I, B> {}

impl<I, B> crate::__seal::Sealed for BorrowedId<'_, I, B> {}
impl<I, B> crate::IsSubject for BorrowedId<'_, I, B> {}
impl<I, B> crate::IsObject for BorrowedId<'_, I, B> {}
impl<I, B> crate::IsGraph for BorrowedId<'_, I, B> {}

impl<'a, I: 'a + ?Sized + ReverseIdInterpretation> Iterator for IdsOf<'a, I> {
    type Item = BorrowedId<'a, I::Iri, I::BlankId>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iris.next().map(BorrowedId::Iri).or_else(|| self.blanks.next().map(BorrowedId::BlankId))
    }
}

pub trait ReverseIdInterpretationMut: ReverseIriInterpretationMut + ReverseBlankIdInterpretationMut {
    fn assign_id_iri(&mut self, r: &Self::Resource, iri: Self::Iri) -> bool {
        self.assign_iri(r, iri)
    }

    fn assign_id_blank(&mut self, r: &Self::Resource, blank: Self::BlankId) -> bool {
        self.assign_blank_id(r, blank)
    }
}

impl<I: ?Sized + ReverseIriInterpretationMut + ReverseBlankIdInterpretationMut> ReverseIdInterpretationMut for I {}
