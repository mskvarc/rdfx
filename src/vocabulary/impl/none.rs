use std::ptr::addr_of_mut;

use crate::{
    BlankId,
    BlankIdBuf,
    Literal,
    LiteralRef,
    vocabulary::{BlankIdVocabulary, BlankIdVocabularyMut, IriVocabulary, IriVocabularyMut, LiteralVocabulary, LiteralVocabularyMut},
};
use iri_rs::{Iri, IriBuf};

/// No vocabulary.
///
/// This is an alias to the unit type.
/// This vocabulary does not store anything.
pub type NoVocabulary = ();

static mut NO_VOCABULARY: NoVocabulary = ();

/// Returns a static reference to unit (no vocabulary).
#[inline(always)]
pub fn no_vocabulary() -> &'static NoVocabulary {
    &()
}

/// Returns a static mutable reference to unit (no vocabulary).
#[inline(always)]
pub fn no_vocabulary_mut() -> &'static mut NoVocabulary {
    unsafe { &mut *addr_of_mut!(NO_VOCABULARY) }
}

impl IriVocabulary for NoVocabulary {
    type Iri = IriBuf;

    fn iri<'i>(&'i self, id: &'i IriBuf) -> Option<Iri<&'i str>> {
        Some(id.as_ref())
    }

    fn owned_iri(&self, id: Self::Iri) -> Result<IriBuf, Self::Iri> {
        Ok(id)
    }

    fn get(&self, iri: Iri<&str>) -> Option<IriBuf> {
        Some(IriBuf::from(iri))
    }
}

impl IriVocabularyMut for NoVocabulary {
    fn insert(&mut self, iri: Iri<&str>) -> IriBuf {
        IriBuf::from(iri)
    }

    fn insert_owned(&mut self, iri: IriBuf) -> Self::Iri {
        iri
    }
}

impl BlankIdVocabulary for NoVocabulary {
    type BlankId = BlankIdBuf;

    fn blank_id<'b>(&'b self, id: &'b BlankIdBuf) -> Option<&'b BlankId> {
        Some(id.as_blank_id_ref())
    }

    fn owned_blank_id(&self, id: Self::BlankId) -> Result<BlankIdBuf, Self::BlankId> {
        Ok(id)
    }

    fn get_blank_id(&self, id: &BlankId) -> Option<BlankIdBuf> {
        Some(id.to_owned())
    }
}

impl BlankIdVocabularyMut for NoVocabulary {
    fn insert_blank_id(&mut self, id: &BlankId) -> BlankIdBuf {
        id.to_owned()
    }

    fn insert_owned_blank_id(&mut self, id: BlankIdBuf) -> Self::BlankId {
        id
    }
}

impl LiteralVocabulary for NoVocabulary {
    type Literal = Literal;

    fn literal<'l>(&'l self, id: &'l Self::Literal) -> Option<LiteralRef<'l>> {
        Some(id.as_ref())
    }

    fn owned_literal(&self, id: Self::Literal) -> Result<Literal, Self::Literal> {
        Ok(id)
    }

    fn get_literal(&self, id: LiteralRef<'_>) -> Option<Self::Literal> {
        Some(id.to_owned())
    }
}

impl LiteralVocabularyMut for NoVocabulary {
    fn insert_literal(&mut self, value: LiteralRef<'_>) -> Self::Literal {
        value.to_owned()
    }

    fn insert_owned_literal(&mut self, value: Literal) -> Self::Literal {
        value
    }
}
