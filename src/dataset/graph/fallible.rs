use crate::{Triple, pattern::CanonicalTriplePattern, utils::InfallibleIterator};

use super::{Graph, GraphMut, PatternMatchingGraph, TraversableGraph};

/// Fallible graph.
pub trait FallibleGraph {
    type Subject: crate::IsSubject;
    type Predicate: crate::IsPredicate;
    type Object: crate::IsObject;
    type Error;
}

impl<D: Graph> FallibleGraph for D {
    type Subject = D::Subject;
    type Predicate = D::Predicate;
    type Object = D::Object;
    type Error = std::convert::Infallible;
}

pub trait FallibleTraversableGraph: FallibleGraph {
    type TryTriples<'a>: Iterator<Item = Result<Triple<&'a Self::Subject, &'a Self::Predicate, &'a Self::Object>, Self::Error>>
    where
        Self: 'a;

    fn try_triples(&self) -> Self::TryTriples<'_>;
}

impl<D: TraversableGraph> FallibleTraversableGraph for D {
    type TryTriples<'a>
        = InfallibleIterator<D::Triples<'a>>
    where
        Self: 'a;

    fn try_triples(&self) -> Self::TryTriples<'_> {
        InfallibleIterator(self.triples())
    }
}

/// Pattern-matching-capable fallible graph. Restricted to uniform graphs
/// whose subject type implements [`crate::Resource`] (so refs serve in any
/// position).
pub trait FalliblePatternMatchingGraph: FallibleGraph
where
    <Self as FallibleGraph>::Subject: crate::Resource,
{
    type TryTriplePatternMatching<'a, 'p>: Iterator<Item = Result<Triple<&'a Self::Subject, &'a Self::Subject, &'a Self::Subject>, Self::Error>>
    where
        Self: 'a,
        Self::Subject: 'p;

    fn try_triple_pattern_matching<'p>(&self, pattern: CanonicalTriplePattern<&'p Self::Subject>) -> Self::TryTriplePatternMatching<'_, 'p>;

    fn try_contains_triple(&self, triple: Triple<&Self::Subject, &Self::Subject, &Self::Subject>) -> Result<bool, Self::Error> {
        Ok(self.try_triple_pattern_matching(triple.into()).next().transpose()?.is_some())
    }
}

impl<D: PatternMatchingGraph> FalliblePatternMatchingGraph for D
where
    <D as Graph>::Subject: crate::Resource,
{
    type TryTriplePatternMatching<'a, 'p>
        = InfallibleIterator<D::TriplePatternMatching<'a, 'p>>
    where
        Self: 'a,
        Self::Subject: 'p;

    fn try_triple_pattern_matching<'p>(&self, pattern: CanonicalTriplePattern<&'p Self::Subject>) -> Self::TryTriplePatternMatching<'_, 'p> {
        InfallibleIterator(self.triple_pattern_matching(pattern))
    }
}

/// Fallible mutable graph.
pub trait FallibleGraphMut: FallibleGraph {
    fn try_insert(&mut self, triple: Triple<Self::Subject, Self::Predicate, Self::Object>) -> Result<(), Self::Error>;
}

impl<D: GraphMut> FallibleGraphMut for D {
    fn try_insert(&mut self, triple: Triple<Self::Subject, Self::Predicate, Self::Object>) -> Result<(), Self::Error> {
        self.insert(triple);
        Ok(())
    }
}
