use iri_rs::{Iri, IriBuf};

use super::{EmbedIntoVocabulary, EmbeddedIntoVocabulary};

/// IRI vocabulary.
pub trait IriVocabulary {
    type Iri;

    /// Returns the IRI associated to the given IRI id.
    fn iri<'i>(&'i self, id: &'i Self::Iri) -> Option<Iri<&'i str>>;

    /// Returns a copy of the IRI associated to the given IRI id.
    fn owned_iri(&self, id: Self::Iri) -> Result<IriBuf, Self::Iri> {
        self.iri(&id).map(IriBuf::from).ok_or(id)
    }

    /// Returns the id of the given IRI, if any.
    fn get(&self, iri: Iri<&str>) -> Option<Self::Iri>;
}

impl<V: IriVocabulary> IriVocabulary for &V {
    type Iri = V::Iri;

    fn iri<'i>(&'i self, id: &'i Self::Iri) -> Option<Iri<&'i str>> {
        V::iri(*self, id)
    }

    fn owned_iri(&self, id: Self::Iri) -> Result<IriBuf, Self::Iri> {
        V::owned_iri(*self, id)
    }

    fn get(&self, iri: Iri<&str>) -> Option<Self::Iri> {
        V::get(*self, iri)
    }
}

impl<V: IriVocabulary> IriVocabulary for &mut V {
    type Iri = V::Iri;

    fn iri<'i>(&'i self, id: &'i Self::Iri) -> Option<Iri<&'i str>> {
        V::iri(*self, id)
    }

    fn owned_iri(&self, id: Self::Iri) -> Result<IriBuf, Self::Iri> {
        V::owned_iri(*self, id)
    }

    fn get(&self, iri: Iri<&str>) -> Option<Self::Iri> {
        V::get(*self, iri)
    }
}

/// Mutable IRI vocabulary.
pub trait IriVocabularyMut: IriVocabulary {
    /// Inserts an IRI to the vocabulary and returns its id.
    ///
    /// If the IRI was already present in the vocabulary, no new id is created
    /// and the current one is returned.
    fn insert(&mut self, iri: Iri<&str>) -> Self::Iri;

    fn insert_owned(&mut self, iri: IriBuf) -> Self::Iri {
        self.insert(iri.as_ref())
    }
}

impl<V: IriVocabularyMut> IriVocabularyMut for &mut V {
    fn insert(&mut self, iri: Iri<&str>) -> Self::Iri {
        V::insert(*self, iri)
    }

    fn insert_owned(&mut self, iri: IriBuf) -> Self::Iri {
        V::insert_owned(*self, iri)
    }
}

impl<V: IriVocabularyMut> EmbedIntoVocabulary<V> for Iri<&str> {
    type Embedded = V::Iri;

    fn embed_into_vocabulary(self, vocabulary: &mut V) -> Self::Embedded {
        vocabulary.insert(self)
    }
}

impl<V: IriVocabularyMut> EmbedIntoVocabulary<V> for IriBuf {
    type Embedded = V::Iri;

    fn embed_into_vocabulary(self, vocabulary: &mut V) -> Self::Embedded {
        vocabulary.insert_owned(self)
    }
}

impl<V: IriVocabularyMut> EmbeddedIntoVocabulary<V> for Iri<&str> {
    type Embedded = V::Iri;

    fn embedded_into_vocabulary(&self, vocabulary: &mut V) -> Self::Embedded {
        vocabulary.insert(*self)
    }
}

impl<V: IriVocabularyMut> EmbeddedIntoVocabulary<V> for IriBuf {
    type Embedded = V::Iri;

    fn embedded_into_vocabulary(&self, vocabulary: &mut V) -> Self::Embedded {
        vocabulary.insert(self.as_ref())
    }
}
