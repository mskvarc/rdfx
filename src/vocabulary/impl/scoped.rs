use crate::{
    BlankId,
    BlankIdBuf,
    Literal,
    LiteralRef,
    vocabulary::{BlankIdVocabulary, BlankIdVocabularyMut, IriVocabulary, IriVocabularyMut, LiteralVocabulary, LiteralVocabularyMut},
};
use iri_rs::Iri;
use std::collections::HashMap;

/// Vocabulary wrapper that helps avoid blank id collisions.
///
/// This is a wrapper behind a vocabulary that can be used to prevent collisions
/// between blank node identifiers added from different sources.
/// Upon insertion, blank node identifier will be prepended by a prefix
/// representing the "scope" of the blank node identifier.
pub struct Scoped<'a, V: BlankIdVocabulary, S> {
    scope: S,
    map: HashMap<BlankIdBuf, V::BlankId>,
    pub(crate) inner: &'a mut V,
}

impl<'a, V: BlankIdVocabulary, S> Scoped<'a, V, S> {
    /// Create a new wrapper around `vocabulary` with the given `scope`.
    pub fn new(vocabulary: &'a mut V, scope: S) -> Self {
        Self {
            scope,
            map: HashMap::new(),
            inner: vocabulary,
        }
    }
}

impl<'a, V: BlankIdVocabulary + IriVocabulary, S> IriVocabulary for Scoped<'a, V, S> {
    type Iri = V::Iri;

    fn iri<'i>(&'i self, id: &'i Self::Iri) -> Option<Iri<&'i str>> {
        self.inner.iri(id)
    }

    fn get(&self, iri: Iri<&str>) -> Option<Self::Iri> {
        self.inner.get(iri)
    }
}

impl<'a, V: BlankIdVocabulary + IriVocabularyMut, S> IriVocabularyMut for Scoped<'a, V, S> {
    fn insert(&mut self, iri: Iri<&str>) -> Self::Iri {
        self.inner.insert(iri)
    }
}

impl<'a, V: BlankIdVocabulary, S> BlankIdVocabulary for Scoped<'a, V, S>
where
    V::BlankId: Clone,
{
    type BlankId = V::BlankId;

    fn blank_id<'b>(&'b self, id: &'b Self::BlankId) -> Option<&'b BlankId> {
        self.inner.blank_id(id)
    }

    fn get_blank_id(&self, id: &BlankId) -> Option<Self::BlankId> {
        self.map.get(id).cloned()
    }
}

impl<'a, V: BlankIdVocabularyMut, S: std::fmt::Display> BlankIdVocabularyMut for Scoped<'a, V, S>
where
    V::BlankId: Clone,
{
    fn insert_blank_id(&mut self, id: &BlankId) -> Self::BlankId {
        match self.get_blank_id(id) {
            Some(id) => id,
            None => {
                let scoped = BlankIdBuf::from_suffix(&format!("{}:{}", self.scope, id.suffix())).unwrap();
                let i = self.inner.insert_blank_id(&scoped);
                self.map.insert(id.to_owned(), i.clone());
                i
            }
        }
    }
}

impl<'a, V: BlankIdVocabulary + LiteralVocabulary, S> LiteralVocabulary for Scoped<'a, V, S> {
    type Literal = V::Literal;

    fn literal<'l>(&'l self, id: &'l Self::Literal) -> Option<LiteralRef<'l>> {
        self.inner.literal(id)
    }

    fn owned_literal(&self, id: Self::Literal) -> Result<Literal, Self::Literal> {
        self.inner.owned_literal(id)
    }

    fn get_literal(&self, literal: LiteralRef<'_>) -> Option<Self::Literal> {
        self.inner.get_literal(literal)
    }
}

impl<'a, V: BlankIdVocabulary + LiteralVocabularyMut, S> LiteralVocabularyMut for Scoped<'a, V, S> {
    fn insert_literal(&mut self, literal: LiteralRef<'_>) -> Self::Literal {
        self.inner.insert_literal(literal)
    }

    fn insert_owned_literal(&mut self, literal: Literal) -> Self::Literal {
        self.inner.insert_owned_literal(literal)
    }
}
