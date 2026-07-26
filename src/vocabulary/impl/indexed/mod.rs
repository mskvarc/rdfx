use educe::Educe;
use std::marker::PhantomData;

use crate::{
    BlankId,
    BlankIdBuf,
    Literal,
    LiteralRef,
    vocabulary::{BlankIdVocabulary, BlankIdVocabularyMut, IriVocabulary, IriVocabularyMut, LiteralVocabulary, LiteralVocabularyMut},
};
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
///
/// `Clone` is derived through `educe` so the index parameters, which are
/// markers, do not pick up a `Clone` bound. Cloning a vocabulary is what
/// lets a parallel consumer hand one copy to each worker.
#[derive(Educe)]
#[educe(Clone)]
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

    /// Number of IRIs interned in this vocabulary.
    ///
    /// Indices handed out by [`IriVocabularyMut::insert`] are dense and
    /// assigned in insertion order, so this doubles as a watermark: every IRI
    /// interned after this call has an index at or above the returned value.
    pub fn iri_count(&self) -> usize {
        self.iri.len()
    }

    /// Number of blank node identifiers interned in this vocabulary.
    ///
    /// Acts as a watermark in the same way as [`Self::iri_count`].
    pub fn blank_id_count(&self) -> usize {
        self.blank_id.len()
    }

    /// Number of literals interned in this vocabulary.
    ///
    /// Acts as a watermark in the same way as [`Self::iri_count`].
    pub fn literal_count(&self) -> usize {
        self.literal.len()
    }

    /// Iterates over the interned IRIs in index order, skipping the first
    /// `start`.
    ///
    /// The `n`-th item yielded is the IRI at index `start + n`; the iterator is
    /// empty when `start` is at or past [`Self::iri_count`]. Paired with a
    /// watermark taken from `iri_count`, this enumerates exactly the IRIs a
    /// vocabulary gained since that watermark — what a consumer needs in order
    /// to merge a forked copy back into the vocabulary it was forked from.
    pub fn iris_from(&self, start: usize) -> impl ExactSizeIterator<Item = &IriBuf> {
        self.iri.iter().skip(start)
    }

    /// Iterates over the interned blank node identifiers in index order,
    /// skipping the first `start`.
    ///
    /// Behaves as [`Self::iris_from`] does for IRIs.
    pub fn blank_ids_from(&self, start: usize) -> impl ExactSizeIterator<Item = &BlankIdBuf> {
        self.blank_id.iter().skip(start)
    }

    /// Iterates over the interned literals in index order, skipping the first
    /// `start`.
    ///
    /// Behaves as [`Self::iris_from`] does for IRIs.
    pub fn literals_from(&self, start: usize) -> impl ExactSizeIterator<Item = &Literal> {
        self.literal.iter().skip(start)
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
            Err(_) => self.blank_id.get_index_of(&blank_id.to_owned()).map(B::from),
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
            Err(_) => self.literal.get_index_of(&literal.to_owned()).map(L::from),
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

#[cfg(test)]
mod tests {
    use super::*;
    use iri_rs::iri;

    fn vocabulary_with_iris(count: usize) -> IndexVocabulary {
        let mut vocabulary = IndexVocabulary::new();
        for index in 0..count {
            let iri = IriBuf::new(format!("https://example.org/i{index}")).expect("valid IRI");
            let _ = vocabulary.insert_owned(iri);
        }
        vocabulary
    }

    #[test]
    fn iri_count_matches_number_of_distinct_iris_interned() {
        let mut vocabulary: IndexVocabulary = IndexVocabulary::new();
        assert_eq!(vocabulary.iri_count(), 0);
        let _ = vocabulary.insert(iri!("https://example.org/a"));
        let _ = vocabulary.insert(iri!("https://example.org/b"));
        assert_eq!(vocabulary.iri_count(), 2);
    }

    #[test]
    fn reinterning_an_existing_iri_does_not_grow_the_vocabulary() {
        let mut vocabulary: IndexVocabulary = IndexVocabulary::new();
        let first = vocabulary.insert(iri!("https://example.org/a"));
        let again = vocabulary.insert(iri!("https://example.org/a"));
        assert_eq!(first, again);
        assert_eq!(vocabulary.iri_count(), 1);
    }

    #[test]
    fn iris_from_a_watermark_yields_exactly_the_iris_added_after_it() {
        let mut vocabulary = vocabulary_with_iris(3);
        let watermark = vocabulary.iri_count();
        let _ = vocabulary.insert(iri!("https://example.org/late-a"));
        let _ = vocabulary.insert(iri!("https://example.org/late-b"));

        let added: Vec<&IriBuf> = vocabulary.iris_from(watermark).collect();
        assert_eq!(added.len(), 2);
        assert_eq!(added[0].as_str(), "https://example.org/late-a");
        assert_eq!(added[1].as_str(), "https://example.org/late-b");
    }

    #[test]
    fn iris_from_yields_items_at_consecutive_indices_starting_at_the_offset() {
        let vocabulary = vocabulary_with_iris(5);
        let start = 2;
        for (offset, iri) in vocabulary.iris_from(start).enumerate() {
            let index = IriIndex::from(start + offset);
            let stored = vocabulary.iri(&index).expect("index within the vocabulary");
            assert_eq!(stored.as_str(), iri.as_str());
        }
    }

    #[test]
    fn iris_from_past_the_end_yields_nothing() {
        let vocabulary = vocabulary_with_iris(2);
        assert_eq!(vocabulary.iris_from(2).count(), 0);
        assert_eq!(vocabulary.iris_from(99).count(), 0);
    }

    #[test]
    fn iris_from_reports_its_length_without_iterating() {
        let vocabulary = vocabulary_with_iris(6);
        assert_eq!(vocabulary.iris_from(0).len(), 6);
        assert_eq!(vocabulary.iris_from(4).len(), 2);
        assert_eq!(vocabulary.iris_from(6).len(), 0);
    }

    #[test]
    fn blank_id_count_and_enumeration_track_insertions() {
        let mut vocabulary: IndexVocabulary = IndexVocabulary::new();
        let first = BlankIdBuf::new("_:a".to_owned()).expect("valid blank id");
        let second = BlankIdBuf::new("_:b".to_owned()).expect("valid blank id");
        let _ = vocabulary.insert_owned_blank_id(first);
        let watermark = vocabulary.blank_id_count();
        let _ = vocabulary.insert_owned_blank_id(second);

        assert_eq!(watermark, 1);
        assert_eq!(vocabulary.blank_id_count(), 2);
        let added: Vec<&BlankIdBuf> = vocabulary.blank_ids_from(watermark).collect();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].as_str(), "_:b");
    }

    #[test]
    fn literal_count_and_enumeration_track_insertions() {
        let mut vocabulary: IndexVocabulary = IndexVocabulary::new();
        let _ = vocabulary.insert_owned_literal(Literal::plain("first"));
        let watermark = vocabulary.literal_count();
        let _ = vocabulary.insert_owned_literal(Literal::plain("second"));

        assert_eq!(watermark, 1);
        assert_eq!(vocabulary.literal_count(), 2);
        let added: Vec<&Literal> = vocabulary.literals_from(watermark).collect();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].value, "second");
    }
}
