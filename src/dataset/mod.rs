//! Dataset traits and implementations.
use crate::{IsGraph, Quad, Resource, pattern::CanonicalQuadPattern};

pub mod fallible;
pub use fallible::FallibleDataset;

mod graph;
pub use graph::{fallible as fallible_graph, *};

mod r#impl;
pub use r#impl::*;

pub mod isomorphism;

/// RDF dataset (graphs + named graphs).
///
/// Per [RDF 1.1 Concepts §4 Datasets][s4], datasets carry quads with optional
/// graph names. The graph-name position is bounded by [`IsGraph`] (IRIs or
/// blank nodes; no literals).
///
/// [s4]: https://www.w3.org/TR/rdf11-concepts/#section-dataset
pub trait Dataset: Graph {
    type Graph: IsGraph;
}

/// Dataset that can be traversed using a provided quad iterator.
pub trait TraversableDataset: Dataset {
    /// Quads iterator.
    type Quads<'a>: Iterator<Item = Quad<&'a Self::Subject, &'a Self::Predicate, &'a Self::Object, &'a <Self as Dataset>::Graph>>
    where
        Self: 'a;

    /// Returns an iterator over the quads of the dataset.
    fn quads(&self) -> Self::Quads<'_>;

    fn quads_count(&self) -> usize {
        self.quads().count()
    }
}


pub trait ResourceTraversableDataset: Dataset {
    type Resources<'a>: Iterator<Item = &'a Self::Subject>
    where
        Self: 'a;

    fn resources(&self) -> Self::Resources<'_>;

    fn resource_count(&self) -> usize {
        self.resources().count()
    }
}


pub trait SubjectTraversableDataset: Dataset {
    type Subjects<'a>: Iterator<Item = &'a Self::Subject>
    where
        Self: 'a;

    fn subjects(&self) -> Self::Subjects<'_>;

    fn subject_count(&self) -> usize {
        self.subjects().count()
    }
}


pub trait PredicateTraversableDataset: Dataset {
    type Predicates<'a>: Iterator<Item = &'a Self::Predicate>
    where
        Self: 'a;

    fn predicates(&self) -> Self::Predicates<'_>;

    fn predicate_count(&self) -> usize {
        self.predicates().count()
    }
}


pub trait ObjectTraversableDataset: Dataset {
    type Objects<'a>: Iterator<Item = &'a Self::Object>
    where
        Self: 'a;

    fn objects(&self) -> Self::Objects<'_>;

    fn object_count(&self) -> usize {
        self.objects().count()
    }
}


pub trait NamedGraphTraversableDataset: Dataset {
    type NamedGraphs<'a>: Iterator<Item = &'a <Self as Dataset>::Graph>
    where
        Self: 'a;

    fn named_graphs(&self) -> Self::NamedGraphs<'_>;

    fn named_graph_count(&self) -> usize {
        self.named_graphs().count()
    }
}


pub trait MultiPatternMatchingDataset: Dataset
where
    <Self as Graph>::Subject: Resource,
    <Self as Dataset>::Graph: PartialEq<<Self as Graph>::Subject>,
{
    /// Pattern-matching iterator.
    type QuadMultiPatternMatching<'a, 'p>: Iterator<Item = Quad<&'a Self::Subject, &'a Self::Subject, &'a Self::Subject, &'a <Self as Dataset>::Graph>>
    where
        Self: 'a,
        Self::Subject: 'p;

    /// Returns an iterator over all the quads of the dataset matching the given
    /// pattern.
    fn quad_multi_pattern_matching<'p, P: IntoIterator<Item = &'p Self::Subject>>(
        &self,
        pattern: CanonicalQuadPattern<P>,
    ) -> Self::QuadMultiPatternMatching<'_, 'p>;
}

/// Pattern-matching-capable dataset. Restricted to uniform datasets where
/// every position carries the same resource type.
pub trait PatternMatchingDataset:
    Dataset<Graph = <Self as Graph>::Subject> + Graph<Predicate = <Self as Graph>::Subject, Object = <Self as Graph>::Subject>
where
    <Self as Graph>::Subject: Resource,
{
    /// Pattern-matching iterator.
    type QuadPatternMatching<'a, 'p>: Iterator<Item = Quad<&'a Self::Subject, &'a Self::Subject, &'a Self::Subject, &'a Self::Subject>>
    where
        Self: 'a,
        Self::Subject: 'p;

    /// Returns an iterator over all the quads of the dataset matching the given
    /// pattern.
    fn quad_pattern_matching<'p>(&self, pattern: CanonicalQuadPattern<&'p Self::Subject>) -> Self::QuadPatternMatching<'_, 'p>;

    /// Checks if the dataset contains the given quad.
    fn contains_quad(&self, quad: Quad<&Self::Subject, &Self::Subject, &Self::Subject, &Self::Subject>) -> bool {
        self.quad_pattern_matching(quad.into()).next().is_some()
    }

    /// Checks if the dataset contains the given subject.
    fn contains_quad_subject(&self, subject: &Self::Subject) -> bool {
        use crate::pattern::quad::canonical::{GivenSubject, GivenSubjectAnyPredicate, GivenSubjectAnyPredicateAnyObject};
        self.quad_pattern_matching(CanonicalQuadPattern::GivenSubject(
            subject,
            GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::AnyObject(GivenSubjectAnyPredicateAnyObject::AnyGraph)),
        ))
        .next()
        .is_some()
    }

    /// Checks if the dataset contains the given predicate.
    fn contains_quad_predicate(&self, predicate: &Self::Subject) -> bool {
        use crate::pattern::quad::canonical::{AnySubject, AnySubjectGivenPredicate, AnySubjectGivenPredicateAnyObject};
        self.quad_pattern_matching(CanonicalQuadPattern::AnySubject(AnySubject::GivenPredicate(
            predicate,
            AnySubjectGivenPredicate::AnyObject(AnySubjectGivenPredicateAnyObject::AnyGraph),
        )))
        .next()
        .is_some()
    }

    /// Checks if the dataset contains the given object.
    fn contains_quad_object(&self, object: &Self::Subject) -> bool {
        use crate::pattern::quad::canonical::{AnySubject, AnySubjectAnyPredicate, AnySubjectAnyPredicateGivenObject};
        self.quad_pattern_matching(CanonicalQuadPattern::AnySubject(AnySubject::AnyPredicate(AnySubjectAnyPredicate::GivenObject(
            object,
            AnySubjectAnyPredicateGivenObject::AnyGraph,
        ))))
        .next()
        .is_some()
    }

    /// Checks if the dataset contains the given named graph.
    fn contains_named_graph(&self, named_graph: &Self::Subject) -> bool {
        use crate::pattern::quad::canonical::{AnySubject, AnySubjectAnyPredicate, AnySubjectAnyPredicateAnyObject};
        self.quad_pattern_matching(CanonicalQuadPattern::AnySubject(AnySubject::AnyPredicate(AnySubjectAnyPredicate::AnyObject(
            AnySubjectAnyPredicateAnyObject::GivenGraph(Some(named_graph)),
        ))))
        .next()
        .is_some()
    }

    /// Returns an iterator over all the predicates `p` matching any quad
    /// `subject p o graph` present in the dataset, for any object `o`.
    fn quad_predicates_objects<'p>(&self, graph: Option<&'p Self::Subject>, subject: &'p Self::Subject) -> QuadPredicatesObjects<'_, 'p, Self>
    where
        Self: PredicateTraversableDataset,
    {
        QuadPredicatesObjects {
            graph,
            subject,
            predicates: self.predicates(),
            dataset: self,
        }
    }

    /// Returns an iterator over all the objects `o` matching the quad `subject predicate o graph`.
    fn quad_objects<'p>(&self, graph: Option<&'p Self::Subject>, subject: &'p Self::Subject, predicate: &'p Self::Subject) -> QuadObjects<'_, 'p, Self> {
        QuadObjects {
            first: None,
            inner: self.quad_pattern_matching(CanonicalQuadPattern::from_option_quad(Quad(Some(subject), Some(predicate), None, Some(graph)))),
        }
    }
}


pub struct QuadPredicatesObjects<'a, 'p, D: 'a + ?Sized + PredicateTraversableDataset + PatternMatchingDataset>
where
    D::Subject: Resource,
{
    graph: Option<&'p D::Subject>,
    subject: &'p D::Subject,
    predicates: D::Predicates<'a>,
    dataset: &'a D,
}

impl<'a: 'p, 'p, D: 'a + ?Sized + PredicateTraversableDataset + PatternMatchingDataset> Iterator for QuadPredicatesObjects<'a, 'p, D>
where
    D::Subject: Resource + 'p,
{
    type Item = (&'a D::Subject, QuadObjects<'p, 'p, D>);

    fn next(&mut self) -> Option<Self::Item> {
        for predicate in &mut self.predicates {
            use crate::pattern::quad::canonical::{GivenSubject, GivenSubjectGivenPredicate, GivenSubjectGivenPredicateAnyObject};
            let pattern = CanonicalQuadPattern::GivenSubject(
                self.subject,
                GivenSubject::GivenPredicate(
                    predicate,
                    GivenSubjectGivenPredicate::AnyObject(GivenSubjectGivenPredicateAnyObject::GivenGraph(self.graph)),
                ),
            );

            let mut iter = self.dataset.quad_pattern_matching(pattern);
            if let Some(Quad(_, _, o, _)) = iter.next() {
                return Some((predicate, QuadObjects { first: Some(o), inner: iter }));
            }
        }

        None
    }
}

pub struct QuadObjects<'a, 'p, D: 'a + ?Sized + PatternMatchingDataset>
where
    D::Subject: Resource + 'p,
{
    first: Option<&'a D::Subject>,
    inner: D::QuadPatternMatching<'a, 'p>,
}

impl<'a, 'p, D: 'a + ?Sized + PatternMatchingDataset> Iterator for QuadObjects<'a, 'p, D>
where
    D::Subject: Resource + 'p,
{
    type Item = &'a D::Subject;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| self.inner.next().map(Quad::into_object))
    }
}

/// Mutable dataset.
pub trait DatasetMut: Dataset {
    /// Inserts the given quad in the dataset.
    fn insert(&mut self, quad: Quad<Self::Subject, Self::Predicate, Self::Object, <Self as Dataset>::Graph>);

    /// Removes the given quad from the dataset.
    fn remove(&mut self, quad: Quad<&Self::Subject, &Self::Predicate, &Self::Object, &<Self as Dataset>::Graph>);
}

/// Dataset view focusing on a given graph.
pub struct DatasetView<'a, D: Dataset> {
    pub dataset: &'a D,
    pub graph: Option<&'a D::Subject>,
}

/// Dataset view focusing on a given resource and restricted to the given graph.
pub struct DatasetGraphView<'a, D: Dataset> {
    pub dataset: &'a D,
    pub graph: Option<&'a D::Subject>,
    pub resource: &'a D::Subject,
}
