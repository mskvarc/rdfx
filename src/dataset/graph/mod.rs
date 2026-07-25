use crate::{IsObject, IsPredicate, IsSubject, Resource, Triple, pattern::CanonicalTriplePattern};

/// Fallible counterparts of the graph traits, for storage that can fail.
pub mod fallible;
pub use fallible::FallibleGraph;

mod r#impl;
pub use r#impl::*;

/// RDF graph.
///
/// Per [RDF 1.1 Concepts §3 Triples][s], terms are typed by their position:
/// subjects must be IRIs or blank nodes (no literals), predicates must be
/// IRIs (no blank nodes or literals), objects may be any term. The three
/// associated types capture this invariant; uniform-resource graphs (where
/// `Subject = Predicate = Object`) automatically satisfy [`UniformGraph<R>`]
/// via blanket impl.
///
/// [s]: https://www.w3.org/TR/rdf11-concepts/#section-triples
pub trait Graph {
    /// Type of the subject.
    type Subject: IsSubject;
    /// Type of the predicate.
    type Predicate: IsPredicate;
    /// Type of the object.
    type Object: IsObject;
}

/// Marker for graphs whose subject/predicate/object positions all carry the
/// same resource type — typical for graphs storing opaque handles.
pub trait UniformGraph<R: Resource>: Graph<Subject = R, Predicate = R, Object = R> {}
impl<R: Resource, G: Graph<Subject = R, Predicate = R, Object = R>> UniformGraph<R> for G {}

/// Graph whose triples can be enumerated.
pub trait TraversableGraph: Graph {
    /// Triples iterator.
    type Triples<'a>: Iterator<Item = Triple<&'a Self::Subject, &'a Self::Predicate, &'a Self::Object>>
    where
        Self: 'a;

    /// Returns an iterator over the triples of this graph.
    fn triples(&self) -> Self::Triples<'_>;

    /// Returns the number of triples in this graph.
    fn triples_count(&self) -> usize {
        self.triples().count()
    }
}

/// Graph whose resources can be enumerated.
pub trait ResourceTraversableGraph: Graph {
    /// Resources iterator.
    type GraphResources<'a>: Iterator<Item = &'a Self::Subject>
    where
        Self: 'a;

    /// Returns an iterator over the resources of this graph.
    fn graph_resources(&self) -> Self::GraphResources<'_>;

    /// Returns the number of resources in this graph.
    fn graph_resource_count(&self) -> usize {
        self.graph_resources().count()
    }
}

/// Graph whose subjects can be enumerated.
pub trait SubjectTraversableGraph: Graph {
    /// Subjects iterator.
    type GraphSubjects<'a>: Iterator<Item = &'a Self::Subject>
    where
        Self: 'a;

    /// Returns an iterator over the subjects of this graph.
    fn graph_subjects(&self) -> Self::GraphSubjects<'_>;

    /// Returns the number of distinct subjects in this graph.
    fn graph_subject_count(&self) -> usize {
        self.graph_subjects().count()
    }
}

/// Graph whose predicates can be enumerated.
pub trait PredicateTraversableGraph: Graph {
    /// Predicates iterator.
    type GraphPredicates<'a>: Iterator<Item = &'a Self::Predicate>
    where
        Self: 'a;

    /// Returns an iterator over the predicates of this graph.
    fn graph_predicates(&self) -> Self::GraphPredicates<'_>;

    /// Returns the number of distinct predicates in this graph.
    fn graph_predicate_count(&self) -> usize {
        self.graph_predicates().count()
    }
}

/// Graph whose objects can be enumerated.
pub trait ObjectTraversableGraph: Graph {
    /// Objects iterator.
    type GraphObjects<'a>: Iterator<Item = &'a Self::Object>
    where
        Self: 'a;

    /// Returns an iterator over the objects of this graph.
    fn graph_objects(&self) -> Self::GraphObjects<'_>;

    /// Returns the number of distinct objects in this graph.
    fn graph_object_count(&self) -> usize {
        self.graph_objects().count()
    }
}

/// Pattern-matching-capable graph. Restricted to uniform graphs whose
/// resource type implements [`Resource`] (granting all four position traits).
pub trait PatternMatchingGraph: Graph<Predicate = <Self as Graph>::Subject, Object = <Self as Graph>::Subject>
where
    <Self as Graph>::Subject: Resource,
{
    /// Pattern-matching iterator.
    type TriplePatternMatching<'a, 'p>: Iterator<Item = Triple<&'a Self::Subject, &'a Self::Subject, &'a Self::Subject>>
    where
        Self: 'a,
        Self::Subject: 'p;

    /// Returns an iterator over the triples matching `pattern`.
    fn triple_pattern_matching<'p>(&self, pattern: CanonicalTriplePattern<&'p Self::Subject>) -> Self::TriplePatternMatching<'_, 'p>;

    /// Checks whether this graph contains `triple`.
    fn contains_triple(&self, triple: Triple<&Self::Subject, &Self::Subject, &Self::Subject>) -> bool {
        self.triple_pattern_matching(triple.into()).next().is_some()
    }

    /// Checks if the graph contains the given subject.
    fn contains_triple_subject(&self, subject: &Self::Subject) -> bool {
        use crate::pattern::triple::canonical::{GivenSubject, GivenSubjectAnyPredicate};
        self.triple_pattern_matching(CanonicalTriplePattern::GivenSubject(
            subject,
            GivenSubject::AnyPredicate(GivenSubjectAnyPredicate::AnyObject),
        ))
        .next()
        .is_some()
    }

    /// Checks if the graph contains the given predicate.
    fn contains_triple_predicate(&self, predicate: &Self::Subject) -> bool {
        use crate::pattern::triple::canonical::{AnySubject, AnySubjectGivenPredicate};
        self.triple_pattern_matching(CanonicalTriplePattern::AnySubject(AnySubject::GivenPredicate(
            predicate,
            AnySubjectGivenPredicate::AnyObject,
        )))
        .next()
        .is_some()
    }

    /// Checks if the graph contains the given object.
    fn contains_triple_object(&self, object: &Self::Subject) -> bool {
        use crate::pattern::triple::canonical::{AnySubject, AnySubjectAnyPredicate};
        self.triple_pattern_matching(CanonicalTriplePattern::AnySubject(AnySubject::AnyPredicate(
            AnySubjectAnyPredicate::GivenObject(object),
        )))
        .next()
        .is_some()
    }

    /// Returns an iterator over all the predicates `p` matching the triple `subject p o` present in the graph, for some `o`.
    fn triple_predicates_objects<'p>(&self, subject: &'p Self::Subject) -> TriplePredicatesObjects<'_, 'p, Self>
    where
        Self: PredicateTraversableGraph,
    {
        TriplePredicatesObjects {
            subject,
            predicates: self.graph_predicates(),
            graph: self,
        }
    }

    /// Returns an iterator over all the objects `o` matching the triple `subject predicate o` present in the graph.
    fn triple_objects<'p>(&self, subject: &'p Self::Subject, predicate: &'p Self::Subject) -> TripleObjects<'_, 'p, Self> {
        TripleObjects {
            first: None,
            inner: self.triple_pattern_matching(CanonicalTriplePattern::from_option_triple(Triple(Some(subject), Some(predicate), None))),
        }
    }
}

/// Iterator over the predicate-object pairs of a subject in a graph.
pub struct TriplePredicatesObjects<'a, 'p, G: 'a + ?Sized + PredicateTraversableGraph + PatternMatchingGraph>
where
    G::Subject: Resource,
{
    subject: &'p G::Subject,
    predicates: G::GraphPredicates<'a>,
    graph: &'a G,
}

impl<'a: 'p, 'p, G: 'a + ?Sized + PredicateTraversableGraph + PatternMatchingGraph> Iterator for TriplePredicatesObjects<'a, 'p, G>
where
    G::Subject: Resource + 'p,
{
    type Item = (&'a G::Subject, TripleObjects<'p, 'p, G>);

    fn next(&mut self) -> Option<Self::Item> {
        for predicate in &mut self.predicates {
            use crate::pattern::triple::canonical::{GivenSubject, GivenSubjectGivenPredicate};
            let pattern = CanonicalTriplePattern::GivenSubject(self.subject, GivenSubject::GivenPredicate(predicate, GivenSubjectGivenPredicate::AnyObject));

            let mut iter = self.graph.triple_pattern_matching(pattern);
            if let Some(Triple(_, _, o)) = iter.next() {
                return Some((predicate, TripleObjects { first: Some(o), inner: iter }));
            }
        }

        None
    }
}

/// Iterator over the objects of a subject-predicate pair in a graph.
pub struct TripleObjects<'a, 'p, D: 'a + ?Sized + PatternMatchingGraph>
where
    D::Subject: Resource + 'p,
{
    first: Option<&'a D::Subject>,
    inner: D::TriplePatternMatching<'a, 'p>,
}

impl<'a, 'p, D: 'a + ?Sized + PatternMatchingGraph> Iterator for TripleObjects<'a, 'p, D>
where
    D::Subject: Resource + 'p,
{
    type Item = &'a D::Subject;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| self.inner.next().map(Triple::into_object))
    }
}

/// Mutable graph.
pub trait GraphMut: Graph {
    /// Inserts `triple` into this graph.
    fn insert(&mut self, triple: Triple<Self::Subject, Self::Predicate, Self::Object>);

    /// Removes `triple` from this graph.
    fn remove(&mut self, triple: Triple<&Self::Subject, &Self::Predicate, &Self::Object>);
}

/// Graph view focusing on a given resource.
pub struct GraphView<'a, G: Graph> {
    /// The graph being viewed.
    pub graph: &'a G,
    /// The resource in focus.
    pub resource: &'a G::Subject,
}
