use std::marker::PhantomData;

use crate::vocabulary::{
    BlankIdVocabulary, BlankIdVocabularyMut, IriVocabulary, IriVocabularyMut, LiteralVocabulary,
    LiteralVocabularyMut,
};
use crate::{BlankId, BlankIdBuf, Literal, LiteralRef};
use indexmap::IndexSet;
use iri_rs::{Iri, IriBuf};

mod blankid;
mod iri;
mod literal;

pub use blankid::*;
pub use iri::*;
pub use literal::*;

/// Vocabulary that stores IRIs and blank node identifiers
/// with a unique index.
pub struct IndexVocabulary<I = IriIndex, B = BlankIdIndex, L = LiteralIndex> {
    iri: IndexSet<IriBuf>,
    blank_id: IndexSet<BlankIdBuf>,
    literal: IndexSet<Literal>,
    ibl: PhantomData<(I, B, L)>,
}

impl<I, B, L> Default for IndexVocabulary<I, B, L> {
    fn default() -> Self {
        Self {
            iri: IndexSet::new(),
            blank_id: IndexSet::new(),
            literal: IndexSet::new(),
            ibl: PhantomData,
        }
    }
}

impl<I, B, L> IndexVocabulary<I, B, L> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<I: IndexedIri, B, L> IriVocabulary for IndexVocabulary<I, B, L> {
    type Iri = I;

    fn iri<'i>(&'i self, id: &'i I) -> Option<Iri<&'i str>> {
        match id.index() {
            IriOrIndex::Iri(iri) => Some(iri),
            IriOrIndex::Index(i) => self.iri.get_index(i).map(|b| b.as_ref()),
        }
    }

    fn get(&self, iri: Iri<&str>) -> Option<I> {
        match I::try_from(iri) {
            Ok(id) => Some(id),
            Err(_) => self.iri.get_index_of(&IriBuf::from(iri)).map(I::from),
        }
    }
}

impl<I: IndexedIri, B, L> IriVocabularyMut for IndexVocabulary<I, B, L> {
    fn insert(&mut self, iri: Iri<&str>) -> I {
        match I::try_from(iri) {
            Ok(id) => id,
            Err(_) => self.iri.insert_full(IriBuf::from(iri)).0.into(),
        }
    }

    fn insert_owned(&mut self, iri: IriBuf) -> Self::Iri {
        if let Ok(id) = I::try_from(iri.as_ref()) {
            return id;
        }

        self.iri.insert_full(iri).0.into()
    }
}

impl<I, B: IndexedBlankId, L> BlankIdVocabulary for IndexVocabulary<I, B, L> {
    type BlankId = B;

    fn blank_id<'b>(&'b self, id: &'b B) -> Option<&'b BlankId> {
        match id.blank_id_index() {
            BlankIdOrIndex::BlankId(id) => Some(id),
            BlankIdOrIndex::Index(i) => self.blank_id.get_index(i).map(BlankIdBuf::as_blank_id_ref),
        }
    }

    fn get_blank_id(&self, blank_id: &BlankId) -> Option<B> {
        match B::try_from(blank_id) {
            Ok(id) => Some(id),
            Err(_) => self
                .blank_id
                .get_index_of(&blank_id.to_owned())
                .map(B::from),
        }
    }
}

impl<I, B: IndexedBlankId, L> BlankIdVocabularyMut for IndexVocabulary<I, B, L> {
    fn insert_blank_id(&mut self, blank_id: &BlankId) -> Self::BlankId {
        match B::try_from(blank_id) {
            Ok(id) => id,
            Err(_) => self.blank_id.insert_full(blank_id.to_owned()).0.into(),
        }
    }

    fn insert_owned_blank_id(&mut self, id: BlankIdBuf) -> Self::BlankId {
        if let Ok(id) = B::try_from(id.as_blank_id_ref()) {
            return id;
        }

        self.blank_id.insert_full(id).0.into()
    }
}

impl<I, B, L: IndexedLiteral> LiteralVocabulary for IndexVocabulary<I, B, L> {
    type Literal = L;

    fn literal<'b>(&'b self, id: &'b L) -> Option<LiteralRef<'b>> {
        match id.literal_index() {
            LiteralOrIndex::Literal(id) => Some(id.as_ref()),
            LiteralOrIndex::Index(i) => self.literal.get_index(i).map(Literal::as_ref),
        }
    }

    fn owned_literal(&self, id: Self::Literal) -> Result<Literal, Self::Literal> {
        match id.into_literal_index() {
            LiteralOrIndex::Literal(id) => Ok(id),
            LiteralOrIndex::Index(i) => match self.literal.get_index(i).cloned() {
                Some(t) => Ok(t),
                None => Err(i.into()),
            },
        }
    }

    fn get_literal(&self, literal: LiteralRef<'_>) -> Option<L> {
        match L::try_from(literal) {
            Ok(id) => Some(id),
            Err(_) => self
                .literal
                .get_index_of(&literal.to_owned())
                .map(L::from),
        }
    }
}

impl<I, B, L: IndexedLiteral> LiteralVocabularyMut for IndexVocabulary<I, B, L> {
    fn insert_literal(&mut self, literal: LiteralRef<'_>) -> Self::Literal {
        match L::try_from(literal) {
            Ok(id) => id,
            Err(_) => self.literal.insert_full(literal.to_owned()).0.into(),
        }
    }

    fn insert_owned_literal(&mut self, literal: Literal) -> Self::Literal {
        match L::try_from(literal) {
            Ok(id) => id,
            Err(literal) => self.literal.insert_full(literal).0.into(),
        }
    }
}
