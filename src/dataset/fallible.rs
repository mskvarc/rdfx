use crate::{IsGraph, Quad, Triple, pattern::CanonicalQuadPattern, utils::InfallibleIterator};

use super::{Dataset, DatasetMut, Graph, PatternMatchingDataset, TraversableDataset};

/// Fallible dataset.
pub trait FallibleDataset {
    type Subject: crate::IsSubject;
    type Predicate: crate::IsPredicate;
    type Object: crate::IsObject;
    type Graph: IsGraph;
    type Error;
}

impl<D: Dataset> FallibleDataset for D {
    type Subject = <D as Graph>::Subject;
    type Predicate = <D as Graph>::Predicate;
    type Object = <D as Graph>::Object;
    type Graph = <D as Dataset>::Graph;
    type Error = std::convert::Infallible;
}

/// Fallible traversable dataset.
pub trait FallibleTraversableDataset: FallibleDataset {
    /// Fallible quads iterator.
    type TryQuads<'a>: Iterator<
        Item = Result<Quad<&'a Self::Subject, &'a Self::Predicate, &'a Self::Object, &'a <Self as FallibleDataset>::Graph>, Self::Error>,
    >
    where
        Self: 'a;

    fn try_quads(&self) -> Self::TryQuads<'_>;
}

impl<D: TraversableDataset> FallibleTraversableDataset for D {
    type TryQuads<'a>
        = InfallibleIterator<D::Quads<'a>>
    where
        Self: 'a;

    fn try_quads(&self) -> Self::TryQuads<'_> {
        InfallibleIterator(self.quads())
    }
}

/// Pattern-matching-capable fallible dataset. Restricted to uniform
/// datasets whose subject type implements [`crate::Resource`].
pub trait FalliblePatternMatchingDataset: FallibleDataset
where
    <Self as FallibleDataset>::Subject: crate::Resource,
    Self: FallibleDataset<Graph = <Self as FallibleDataset>::Subject>,
{
    type TryQuadPatternMatching<'a, 'p>: Iterator<Item = Result<Quad<&'a Self::Subject, &'a Self::Subject, &'a Self::Subject, &'a Self::Subject>, Self::Error>>
    where
        Self: 'a,
        Self::Subject: 'p;

    fn try_quad_pattern_matching<'p>(&self, pattern: CanonicalQuadPattern<&'p Self::Subject>) -> Self::TryQuadPatternMatching<'_, 'p>;

    fn try_contains_triple(&self, triple: Triple<&Self::Subject, &Self::Subject, &Self::Subject>) -> Result<bool, Self::Error> {
        Ok(self.try_quad_pattern_matching(triple.into()).next().transpose()?.is_some())
    }
}

impl<D: PatternMatchingDataset> FalliblePatternMatchingDataset for D
where
    <D as crate::dataset::Graph>::Subject: crate::Resource,
{
    type TryQuadPatternMatching<'a, 'p>
        = InfallibleIterator<D::QuadPatternMatching<'a, 'p>>
    where
        Self: 'a,
        Self::Subject: 'p;

    fn try_quad_pattern_matching<'p>(&self, pattern: CanonicalQuadPattern<&'p Self::Subject>) -> Self::TryQuadPatternMatching<'_, 'p> {
        InfallibleIterator(self.quad_pattern_matching(pattern))
    }
}

/// Fallible mutable dataset.
pub trait FallibleDatasetMut: FallibleDataset {
    fn try_insert(&mut self, quad: Quad<Self::Subject, Self::Predicate, Self::Object, <Self as FallibleDataset>::Graph>) -> Result<(), Self::Error>;
}

impl<D: DatasetMut> FallibleDatasetMut for D {
    fn try_insert(&mut self, quad: Quad<Self::Subject, Self::Predicate, Self::Object, <Self as FallibleDataset>::Graph>) -> Result<(), Self::Error> {
        self.insert(quad);
        Ok(())
    }
}
