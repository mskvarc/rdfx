use std::{
    cmp::Ordering,
    collections::BTreeSet,
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

use hashbrown::{DefaultHashBuilder, HashSet, HashTable};
use slab::Slab;

use super::{
    super::{Dataset, PatternMatchingDataset},
    HashDataset,
};
use crate::{
    BlankIdBuf,
    LocalTerm,
    Quad,
    RdfDisplay,
    dataset::{
        DatasetMut,
        HashGraph,
        MultiPatternMatchingDataset,
        NamedGraphTraversableDataset,
        ObjectTraversableDataset,
        PredicateTraversableDataset,
        ResourceTraversableDataset,
        SubjectTraversableDataset,
        TraversableDataset,
        isomorphism::{Bijection, are_isomorphic, find_bijection},
    },
    pattern::{
        CanonicalQuadPattern,
        quad::canonical::{PatternGraph, PatternObject, PatternPredicate, PatternSubject},
    },
};

fn quad_with_resources<R>(resources: &Slab<Resource<R>>, Quad(s, p, o, g): Quad<usize>) -> Quad<&R> {
    unsafe {
        Quad(
            &resources.get_unchecked(s).value,
            &resources.get_unchecked(p).value,
            &resources.get_unchecked(o).value,
            g.map(|g| &resources.get_unchecked(g).value),
        )
    }
}

/// Indexed hash-based RDF dataset, optimized for pattern matching operations.
#[derive(Clone)]
pub struct IndexedHashDataset<R = LocalTerm> {
    resources: Slab<Resource<R>>,
    quads: Slab<Quad<usize>>,
    hasher: DefaultHashBuilder,
    resources_indexes: HashTable<usize>,
    quads_indexes: HashTable<usize>,
    subjects: HashSet<usize>,
    predicates: HashSet<usize>,
    objects: HashSet<usize>,
    default_graph: BTreeSet<usize>,
    named_graphs: HashSet<usize>,
}

impl<R> Default for IndexedHashDataset<R> {
    fn default() -> Self {
        Self {
            quads: Slab::new(),
            resources: Slab::new(),
            hasher: DefaultHashBuilder::default(),
            quads_indexes: HashTable::new(),
            resources_indexes: HashTable::new(),
            default_graph: BTreeSet::new(),
            subjects: HashSet::default(),
            predicates: HashSet::default(),
            objects: HashSet::default(),
            named_graphs: HashSet::default(),
        }
    }
}

impl<R> IndexedHashDataset<R> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.quads.len()
    }

    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
    }

    pub fn iter(&self) -> Quads<'_, R> {
        Quads {
            resources: &self.resources,
            quads: &self.quads,
            indexes: self.quads_indexes.iter(),
        }
    }

    pub fn resources(&self) -> Resources<'_, R> {
        Resources {
            resources: &self.resources,
            indexes: self.resources_indexes.iter(),
        }
    }

    pub fn subjects(&self) -> Subjects<'_, R> {
        Subjects {
            resources: &self.resources,
            indexes: self.subjects.iter(),
        }
    }

    pub fn predicates(&self) -> Predicates<'_, R> {
        Predicates {
            resources: &self.resources,
            indexes: self.predicates.iter(),
        }
    }

    pub fn objects(&self) -> Objects<'_, R> {
        Objects {
            resources: &self.resources,
            indexes: self.objects.iter(),
        }
    }

    pub fn named_graphs(&self) -> NamedGraphs<'_, R> {
        NamedGraphs {
            resources: &self.resources,
            indexes: self.named_graphs.iter(),
        }
    }
}

impl<R: Eq + Hash> IndexedHashDataset<R> {
    pub fn from_non_indexed(dataset: HashDataset<R>) -> Self {
        let mut resources: Slab<Resource<R>> = dataset
            .resources
            .into_iter()
            .map(|(i, r)| {
                (
                    i,
                    Resource {
                        value: r.value,
                        as_subject: BTreeSet::new(),
                        as_predicate: BTreeSet::new(),
                        as_object: BTreeSet::new(),
                        as_graph: BTreeSet::new(),
                    },
                )
            })
            .collect();

        let mut subjects = HashSet::default();
        let mut predicates = HashSet::default();
        let mut objects = HashSet::default();
        let mut named_graphs = HashSet::default();
        let mut default_graph = BTreeSet::new();

        for &i in &dataset.quads_indexes {
            let Quad(s, p, o, g) = dataset.quads[i];

            resources[s].as_subject.insert(i);
            subjects.insert(s);

            resources[p].as_predicate.insert(i);
            predicates.insert(p);

            resources[o].as_object.insert(i);
            objects.insert(o);

            match g {
                Some(g) => {
                    resources[g].as_graph.insert(i);
                    named_graphs.insert(g);
                }
                None => {
                    default_graph.insert(i);
                }
            }
        }

        Self {
            resources,
            quads: dataset.quads,
            hasher: dataset.hasher,
            resources_indexes: dataset.resources_indexes,
            quads_indexes: dataset.quads_indexes,
            default_graph,
            subjects,
            predicates,
            objects,
            named_graphs,
        }
    }

    fn hash_resource(&self, r: &R) -> u64 {
        self.hasher.hash_one(r)
    }

    fn hash_quad(&self, q: &Quad<&R>) -> u64 {
        self.hasher.hash_one(q)
    }

    fn index_of_resource(&self, r: &R) -> Option<usize> {
        let h = self.hash_resource(r);
        let resources = &self.resources;
        self.resources_indexes.find(h, |&i| &resources[i].value == r).copied()
    }

    fn get_resource(&self, r: &R) -> Option<&Resource<R>> {
        self.resources.get(self.index_of_resource(r)?)
    }

    fn index_of_quad(&self, q: Quad<&R>) -> Option<usize> {
        let h = self.hash_quad(&q);
        let resources = &self.resources;
        let quads = &self.quads;
        self.quads_indexes.find(h, |&i| quad_with_resources(resources, quads[i]) == q).copied()
    }

    pub fn contains_resource(&self, r: &R) -> bool {
        self.index_of_resource(r).is_some()
    }

    pub fn contains(&self, q: Quad<&R>) -> bool {
        self.index_of_quad(q).is_some()
    }

    pub fn pattern_matching(&self, pattern: CanonicalQuadPattern<&R>) -> PatternMatching<'_, R> {
        PatternMatching {
            resources: &self.resources,
            quads: &self.quads,
            subject: SubjectConstraints::new(self, pattern.into_subject()),
            predicate: PredicateConstraints::new(self, pattern.into_predicate()),
            object: ObjectConstraints::new(self, pattern.into_object()),
            graph: GraphConstraints::new(self, pattern.into_graph()),
            i: 0,
        }
    }

    pub fn multi_pattern_matching<'a, P>(&self, pattern: CanonicalQuadPattern<P>) -> MultiPatternMatching<'_, R>
    where
        P: IntoIterator<Item = &'a R>,
        R: 'a,
    {
        let (s, p, o, g) = pattern.into_parts();

        MultiPatternMatching {
            resources: &self.resources,
            quads: &self.quads,
            subject: SubjectConstraints::new_multi(self, s),
            predicate: PredicateConstraints::new_multi(self, p),
            object: ObjectConstraints::new_multi(self, o),
            graph: GraphConstraints::new_multi(self, g),
            i: 0,
        }
    }
}

impl<R: Clone + Eq + Hash> IndexedHashDataset<R> {
    fn intern_resource(&mut self, r: R, pre: Option<usize>, mk: impl FnOnce(R, usize) -> Resource<R>, i: usize) -> usize {
        match pre {
            Some(j) => j,
            None => {
                let h = self.hasher.hash_one(&r);
                let j = self.resources.insert(mk(r, i));
                let hasher = self.hasher.clone();
                let resources = &self.resources;
                self.resources_indexes.insert_unique(h, j, |&k| hasher.hash_one(&resources[k].value));
                j
            }
        }
    }

    pub fn insert(&mut self, quad: Quad<R>) -> bool {
        if self.contains(quad.as_ref()) {
            return false;
        }

        let s_pre = self.index_of_resource(&quad.0);
        let p_pre = self.index_of_resource(&quad.1);
        let o_pre = self.index_of_resource(&quad.2);
        let g_pre = quad.3.as_ref().map(|g| self.index_of_resource(g));

        let i = self.quads.vacant_key();

        let s_i = self.intern_resource(quad.0, s_pre, Resource::subject, i);
        if s_pre.is_some() {
            self.resources[s_i].as_subject.insert(i);
        }

        let p_i = self.intern_resource(quad.1, p_pre, Resource::predicate, i);
        if p_pre.is_some() {
            self.resources[p_i].as_predicate.insert(i);
        }

        let o_i = self.intern_resource(quad.2, o_pre, Resource::object, i);
        if o_pre.is_some() {
            self.resources[o_i].as_object.insert(i);
        }

        let g_i = match (quad.3, g_pre) {
            (Some(g), Some(pre)) => {
                let g_i = self.intern_resource(g, pre, Resource::graph, i);
                if pre.is_some() {
                    self.resources[g_i].as_graph.insert(i);
                }
                Some(g_i)
            }
            (None, None) => {
                self.default_graph.insert(i);
                None
            }
            _ => unreachable!(),
        };

        self.subjects.insert(s_i);
        self.predicates.insert(p_i);
        self.objects.insert(o_i);
        if let Some(g_i) = g_i {
            self.named_graphs.insert(g_i);
        }

        let stored = Quad(s_i, p_i, o_i, g_i);
        let inserted = self.quads.insert(stored);
        debug_assert_eq!(inserted, i);

        let h = self.hasher.hash_one(&quad_with_resources(&self.resources, stored));
        let hasher = self.hasher.clone();
        let resources = &self.resources;
        let quads = &self.quads;
        self.quads_indexes
            .insert_unique(h, i, |&j| hasher.hash_one(&quad_with_resources(resources, quads[j])));

        true
    }

    pub fn remove(&mut self, q: Quad<&R>) -> bool {
        let h = self.hash_quad(&q);
        let resources = &self.resources;
        let quads = &self.quads;
        let entry = self.quads_indexes.find_entry(h, |&i| quad_with_resources(resources, quads[i]) == q);

        let i = match entry {
            Ok(e) => {
                let (i, _) = e.remove();
                i
            }
            Err(_) => return false,
        };

        self.remove_by_index(i, false);
        true
    }

    pub fn remove_graph(&mut self, graph: Option<&R>) -> Option<HashGraph<R>>
    where
        R: Clone,
    {
        let indexes: Vec<usize> = match graph {
            Some(g) => {
                let g_i = self.index_of_resource(g)?;
                if self.named_graphs.contains(&g_i) {
                    self.resources[g_i].as_graph.iter().copied().collect()
                } else {
                    return None;
                }
            }
            None => self.default_graph.iter().copied().collect(),
        };

        let mut graph_out = HashGraph::new();
        for i in indexes {
            let quad = quad_with_resources(&self.resources, self.quads[i]).cloned();
            self.remove_by_index(i, true);
            graph_out.insert(quad.into_triple().0);
        }

        Some(graph_out)
    }

    fn remove_resource_index(&mut self, ri: usize) {
        if self.resources[ri].is_empty() {
            let h = self.hash_resource(&self.resources[ri].value);
            if let Ok(entry) = self.resources_indexes.find_entry(h, |&j| j == ri) {
                entry.remove();
            }
            self.resources.remove(ri);
        }
    }

    fn remove_by_index(&mut self, i: usize, remove_index: bool) {
        if remove_index {
            let resources = &self.resources;
            let quads = &self.quads;
            let q = quad_with_resources(resources, quads[i]);
            let h = self.hasher.hash_one(&q);
            if let Ok(entry) = self.quads_indexes.find_entry(h, |&j| j == i) {
                entry.remove();
            }
        }

        let Quad(s_i, p_i, o_i, g_i) = self.quads.remove(i);

        self.subjects.remove(&s_i);
        self.predicates.remove(&p_i);
        self.objects.remove(&o_i);

        self.resources[s_i].as_subject.remove(&i);
        self.remove_resource_index(s_i);
        self.resources[p_i].as_predicate.remove(&i);
        self.remove_resource_index(p_i);
        self.resources[o_i].as_object.remove(&i);
        self.remove_resource_index(o_i);

        match g_i {
            Some(g_i) => {
                self.resources[g_i].as_graph.remove(&i);
                if self.resources[g_i].is_empty() {
                    self.named_graphs.remove(&g_i);
                }
                self.remove_resource_index(g_i);
            }
            None => {
                self.default_graph.remove(&i);
            }
        }
    }

    pub fn extract_pattern_matching(&mut self, pattern: CanonicalQuadPattern<&R>) -> ExtractPatternMatching<'_, R> {
        let subject = SubjectConstraints::new(self, pattern.into_subject());
        let predicate = PredicateConstraints::new(self, pattern.into_predicate());
        let object = ObjectConstraints::new(self, pattern.into_object());
        let graph = GraphConstraints::new(self, pattern.into_graph());

        ExtractPatternMatching {
            dataset: self,
            subject,
            predicate,
            object,
            graph,
            i: 0,
        }
    }
}

impl<R: Clone + Eq + Hash> From<HashDataset<R>> for IndexedHashDataset<R> {
    fn from(value: HashDataset<R>) -> Self {
        Self::from_non_indexed(value)
    }
}

impl<R: Clone + Eq + Hash> FromIterator<Quad<R>> for IndexedHashDataset<R> {
    fn from_iter<T: IntoIterator<Item = Quad<R>>>(iter: T) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl<R: Clone + Eq + Hash> Extend<Quad<R>> for IndexedHashDataset<R> {
    fn extend<T: IntoIterator<Item = Quad<R>>>(&mut self, iter: T) {
        for quad in iter {
            self.insert(quad);
        }
    }
}

impl<R: Clone + Eq + Hash> IndexedHashDataset<R> {
    /// Inserts every quad of `iter` in the dataset.
    pub fn absorb<I: IntoIterator<Item = Quad<R>>>(&mut self, iter: I) {
        self.extend(iter);
    }

    /// Removes the given quad and returns it if it was present.
    pub fn take(&mut self, quad: Quad<&R>) -> Option<Quad<R>> {
        let i = self.index_of_quad(quad)?;
        let value = quad_with_resources(&self.resources, self.quads[i]).cloned();
        self.remove_by_index(i, true);
        Some(value)
    }

    /// Removes and returns the first quad matching the given option-pattern, if any.
    pub fn take_match(&mut self, pattern: Quad<Option<&R>, Option<&R>, Option<&R>, Option<&R>>) -> Option<Quad<R>> {
        let canonical = CanonicalQuadPattern::from_option_quad(pattern);
        self.extract_pattern_matching(canonical).next()
    }
}

impl IndexedHashDataset<LocalTerm> {
    /// Returns a blank-node bijection from `self` to `other` if one exists.
    pub fn find_blank_id_bijection<'a, 'b>(&'a self, other: &'b Self) -> Option<Bijection<'a, 'b>> {
        find_bijection(self, other)
    }

    /// Checks if `self` and `other` are isomorphic.
    pub fn is_isomorphic_to(&self, other: &Self) -> bool {
        are_isomorphic(self, other)
    }

    /// Returns a copy of `self` with each blank node identifier replaced by the
    /// result of `f`.
    pub fn substitute_blank_ids(self, mut f: impl FnMut(BlankIdBuf) -> BlankIdBuf) -> Self {
        self.into_iter()
            .map(|q| {
                q.map(|t| match t {
                    LocalTerm::Anonymous(b) => LocalTerm::Anonymous(f(b)),
                    named => named,
                })
            })
            .collect::<crate::dataset::HashDataset<LocalTerm>>()
            .into_indexed()
    }
}

impl<R> Dataset for IndexedHashDataset<R> {
    type Resource = R;
}

impl<R> TraversableDataset for IndexedHashDataset<R> {
    type Quads<'a>
        = Quads<'a, R>
    where
        R: 'a;

    fn quads(&self) -> Self::Quads<'_> {
        self.iter()
    }
}

impl<R> ResourceTraversableDataset for IndexedHashDataset<R> {
    type Resources<'a>
        = Resources<'a, R>
    where
        R: 'a;

    fn resources(&self) -> Self::Resources<'_> {
        self.resources()
    }

    fn resource_count(&self) -> usize {
        self.resources.len()
    }
}

impl<R> SubjectTraversableDataset for IndexedHashDataset<R> {
    type Subjects<'a>
        = Subjects<'a, R>
    where
        R: 'a;

    fn subjects(&self) -> Self::Subjects<'_> {
        self.subjects()
    }

    fn subject_count(&self) -> usize {
        self.subjects.len()
    }
}

impl<R> PredicateTraversableDataset for IndexedHashDataset<R> {
    type Predicates<'a>
        = Predicates<'a, R>
    where
        R: 'a;

    fn predicates(&self) -> Self::Predicates<'_> {
        self.predicates()
    }

    fn predicate_count(&self) -> usize {
        self.predicates.len()
    }
}

impl<R> ObjectTraversableDataset for IndexedHashDataset<R> {
    type Objects<'a>
        = Objects<'a, R>
    where
        R: 'a;

    fn objects(&self) -> Self::Objects<'_> {
        self.objects()
    }

    fn object_count(&self) -> usize {
        self.objects.len()
    }
}

impl<R> NamedGraphTraversableDataset for IndexedHashDataset<R> {
    type NamedGraphs<'a>
        = NamedGraphs<'a, R>
    where
        R: 'a;

    fn named_graphs(&self) -> Self::NamedGraphs<'_> {
        self.named_graphs()
    }

    fn named_graph_count(&self) -> usize {
        self.named_graphs.len()
    }
}

impl<R: Clone + Eq + Hash> DatasetMut for IndexedHashDataset<R> {
    fn insert(&mut self, quad: Quad<Self::Resource>) {
        self.insert(quad);
    }

    fn remove(&mut self, quad: Quad<&Self::Resource>) {
        self.remove(quad);
    }
}

impl<R: Eq + Hash> PatternMatchingDataset for IndexedHashDataset<R> {
    type QuadPatternMatching<'a, 'p>
        = PatternMatching<'a, R>
    where
        R: 'a,
        Self::Resource: 'p;

    fn quad_pattern_matching<'p>(&self, pattern: CanonicalQuadPattern<&'p Self::Resource>) -> Self::QuadPatternMatching<'_, 'p> {
        self.pattern_matching(pattern)
    }

    fn contains_quad(&self, quad: Quad<&Self::Resource>) -> bool {
        self.contains(quad)
    }
}

impl<R: Eq + Hash> MultiPatternMatchingDataset for IndexedHashDataset<R> {
    type QuadMultiPatternMatching<'a, 'p>
        = MultiPatternMatching<'a, R>
    where
        R: 'a,
        Self::Resource: 'p;

    fn quad_multi_pattern_matching<'p, P>(&self, pattern: CanonicalQuadPattern<P>) -> Self::QuadMultiPatternMatching<'_, 'p>
    where
        P: IntoIterator<Item = &'p R>,
        R: 'p,
    {
        self.multi_pattern_matching(pattern)
    }
}

pub struct Quads<'a, R> {
    resources: &'a Slab<Resource<R>>,
    quads: &'a Slab<Quad<usize>>,
    indexes: hashbrown::hash_table::Iter<'a, usize>,
}

impl<'a, R> Iterator for Quads<'a, R> {
    type Item = Quad<&'a R>;

    fn next(&mut self) -> Option<Self::Item> {
        self.indexes.next().map(|&i| quad_with_resources(self.resources, self.quads[i]))
    }
}

pub struct IntoQuads<R> {
    resources: Slab<Resource<R>>,
    quads: Slab<Quad<usize>>,
    indexes: hashbrown::hash_table::IntoIter<usize>,
}

impl<R: Clone> Iterator for IntoQuads<R> {
    type Item = Quad<R>;

    fn next(&mut self) -> Option<Self::Item> {
        self.indexes.next().map(|i| quad_with_resources(&self.resources, self.quads.remove(i)).cloned())
    }
}

impl<'a, R> IntoIterator for &'a IndexedHashDataset<R> {
    type Item = Quad<&'a R>;
    type IntoIter = Quads<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<R: Clone> IntoIterator for IndexedHashDataset<R> {
    type Item = Quad<R>;
    type IntoIter = IntoQuads<R>;

    fn into_iter(self) -> Self::IntoIter {
        IntoQuads {
            resources: self.resources,
            quads: self.quads,
            indexes: self.quads_indexes.into_iter(),
        }
    }
}

pub struct Resources<'a, R> {
    resources: &'a Slab<Resource<R>>,
    indexes: hashbrown::hash_table::Iter<'a, usize>,
}

impl<'a, R> Iterator for Resources<'a, R> {
    type Item = &'a R;

    fn next(&mut self) -> Option<Self::Item> {
        self.indexes.next().map(|&i| &self.resources[i].value)
    }
}

pub struct Subjects<'a, R> {
    resources: &'a Slab<Resource<R>>,
    indexes: hashbrown::hash_set::Iter<'a, usize>,
}

impl<'a, R> Iterator for Subjects<'a, R> {
    type Item = &'a R;

    fn next(&mut self) -> Option<Self::Item> {
        self.indexes.next().map(|&i| &self.resources[i].value)
    }
}

pub struct Predicates<'a, R> {
    resources: &'a Slab<Resource<R>>,
    indexes: hashbrown::hash_set::Iter<'a, usize>,
}

impl<'a, R> Iterator for Predicates<'a, R> {
    type Item = &'a R;

    fn next(&mut self) -> Option<Self::Item> {
        self.indexes.next().map(|&i| &self.resources[i].value)
    }
}

pub struct Objects<'a, R> {
    resources: &'a Slab<Resource<R>>,
    indexes: hashbrown::hash_set::Iter<'a, usize>,
}

impl<'a, R> Iterator for Objects<'a, R> {
    type Item = &'a R;

    fn next(&mut self) -> Option<Self::Item> {
        self.indexes.next().map(|&i| &self.resources[i].value)
    }
}

pub struct NamedGraphs<'a, R> {
    resources: &'a Slab<Resource<R>>,
    indexes: hashbrown::hash_set::Iter<'a, usize>,
}

impl<'a, R> Iterator for NamedGraphs<'a, R> {
    type Item = &'a R;

    fn next(&mut self) -> Option<Self::Item> {
        self.indexes.next().map(|&i| &self.resources[i].value)
    }
}

impl<R: Eq + Hash> PartialEq for IndexedHashDataset<R> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().all(|q| other.contains(q))
    }
}

impl<R: Eq + Hash> Eq for IndexedHashDataset<R> {}

pub struct PatternMatching<'a, R> {
    resources: &'a Slab<Resource<R>>,
    quads: &'a Slab<Quad<usize>>,
    subject: SubjectConstraints,
    predicate: PredicateConstraints,
    object: ObjectConstraints,
    graph: GraphConstraints,
    i: usize,
}

impl<'a, R> Iterator for PatternMatching<'a, R> {
    type Item = Quad<&'a R>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.quads.capacity() {
            let i = self.subject.next(self.i)?;
            let quad = *self.quads.get(i)?;
            match self.predicate.next(i, quad) {
                Ok(()) => match self.object.next(i, quad) {
                    Ok(()) => match self.graph.next(i, quad) {
                        Ok(()) => {
                            self.i = i + 1;
                            return Some(quad_with_resources(self.resources, quad));
                        }
                        Err(j) => self.i = j?,
                    },
                    Err(j) => self.i = j?,
                },
                Err(j) => self.i = j?,
            }
        }

        None
    }
}

pub struct MultiPatternMatching<'a, R> {
    resources: &'a Slab<Resource<R>>,
    quads: &'a Slab<Quad<usize>>,
    subject: SubjectConstraints,
    predicate: PredicateConstraints,
    object: ObjectConstraints,
    graph: GraphConstraints,
    i: usize,
}

impl<'a, R> Iterator for MultiPatternMatching<'a, R> {
    type Item = Quad<&'a R>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.quads.capacity() {
            let i = self.subject.next(self.i)?;
            let quad = *self.quads.get(i)?;
            match self.predicate.next(i, quad) {
                Ok(()) => match self.object.next(i, quad) {
                    Ok(()) => match self.graph.next(i, quad) {
                        Ok(()) => {
                            self.i = i + 1;
                            return Some(quad_with_resources(self.resources, quad));
                        }
                        Err(j) => self.i = j?,
                    },
                    Err(j) => self.i = j?,
                },
                Err(j) => self.i = j?,
            }
        }

        None
    }
}

pub struct ExtractPatternMatching<'a, R> {
    dataset: &'a mut IndexedHashDataset<R>,
    subject: SubjectConstraints,
    predicate: PredicateConstraints,
    object: ObjectConstraints,
    graph: GraphConstraints,
    i: usize,
}

impl<R: Clone + Eq + Hash> Iterator for ExtractPatternMatching<'_, R> {
    type Item = Quad<R>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.dataset.quads.capacity() {
            let i = self.subject.next(self.i)?;
            let quad = *self.dataset.quads.get(i)?;
            match self.predicate.next(i, quad) {
                Ok(()) => match self.object.next(i, quad) {
                    Ok(()) => match self.graph.next(i, quad) {
                        Ok(()) => {
                            let value = quad_with_resources(&self.dataset.resources, quad).cloned();
                            self.dataset.remove_by_index(i, true);
                            self.i = i + 1;
                            return Some(value);
                        }
                        Err(j) => self.i = j?,
                    },
                    Err(j) => self.i = j?,
                },
                Err(j) => self.i = j?,
            }
        }

        None
    }
}

type SortedIndexes = std::iter::Peekable<std::vec::IntoIter<usize>>;

fn sorted_set(set: &BTreeSet<usize>) -> SortedIndexes {
    set.iter().copied().collect::<Vec<_>>().into_iter().peekable()
}

enum SubjectConstraints {
    None,
    Any,
    Fixed(SortedIndexes),
}

impl SubjectConstraints {
    fn new<R: Eq + Hash>(dataset: &IndexedHashDataset<R>, s: PatternSubject<&R>) -> Self {
        match s {
            PatternSubject::Any => Self::Any,
            PatternSubject::Given(s) => match dataset.get_resource(s) {
                Some(subject) => Self::Fixed(sorted_set(&subject.as_subject)),
                None => Self::None,
            },
        }
    }

    fn new_multi<'p, R, P>(dataset: &IndexedHashDataset<R>, s: PatternSubject<P>) -> Self
    where
        P: IntoIterator<Item = &'p R>,
        R: 'p + Eq + Hash,
    {
        match s {
            PatternSubject::Any => Self::Any,
            PatternSubject::Given(multi_s) => {
                let mut indexes = Vec::new();
                for s in multi_s {
                    if let Some(subject) = dataset.get_resource(s) {
                        indexes.extend(subject.as_subject.iter().copied());
                    }
                }

                if indexes.is_empty() {
                    Self::None
                } else {
                    indexes.sort_unstable();
                    Self::Fixed(indexes.into_iter().peekable())
                }
            }
        }
    }

    fn next(&mut self, i: usize) -> Option<usize> {
        match self {
            Self::None => None,
            Self::Any => Some(i),
            Self::Fixed(indexes) => {
                while let Some(j) = indexes.peek().copied() {
                    if j >= i {
                        return Some(j);
                    }

                    indexes.next();
                }

                None
            }
        }
    }
}

enum PredicateConstraints {
    None,
    Any,
    SameAsSubject,
    Fixed(SortedIndexes),
}

impl PredicateConstraints {
    fn new<R: Eq + Hash>(dataset: &IndexedHashDataset<R>, p: PatternPredicate<&R>) -> Self {
        match p {
            PatternPredicate::Any => Self::Any,
            PatternPredicate::SameAsSubject => Self::SameAsSubject,
            PatternPredicate::Given(s) => match dataset.get_resource(s) {
                Some(predicate) => Self::Fixed(sorted_set(&predicate.as_predicate)),
                None => Self::None,
            },
        }
    }

    fn new_multi<'p, R, P>(dataset: &IndexedHashDataset<R>, p: PatternPredicate<P>) -> Self
    where
        P: IntoIterator<Item = &'p R>,
        R: 'p + Eq + Hash,
    {
        match p {
            PatternPredicate::Any => Self::Any,
            PatternPredicate::SameAsSubject => Self::SameAsSubject,
            PatternPredicate::Given(multi_p) => {
                let mut indexes = Vec::new();
                for p in multi_p {
                    if let Some(predicate) = dataset.get_resource(p) {
                        indexes.extend(predicate.as_predicate.iter().copied());
                    }
                }

                if indexes.is_empty() {
                    Self::None
                } else {
                    indexes.sort_unstable();
                    Self::Fixed(indexes.into_iter().peekable())
                }
            }
        }
    }

    fn next(&mut self, i: usize, quad: Quad<usize>) -> Result<(), Option<usize>> {
        match self {
            Self::None => Err(None),
            Self::Any => Ok(()),
            Self::SameAsSubject => {
                if quad.0 == quad.1 {
                    Ok(())
                } else {
                    Err(i.checked_add(1))
                }
            }
            Self::Fixed(indexes) => {
                while let Some(j) = indexes.peek().copied() {
                    match j.cmp(&i) {
                        Ordering::Equal => return Ok(()),
                        Ordering::Greater => return Err(Some(j)),
                        Ordering::Less => {
                            indexes.next();
                        }
                    }
                }

                Err(None)
            }
        }
    }
}

enum ObjectConstraints {
    None,
    Any,
    SameAsSubject,
    SameAsPredicate,
    Fixed(SortedIndexes),
}

impl ObjectConstraints {
    fn new<R: Eq + Hash>(dataset: &IndexedHashDataset<R>, o: PatternObject<&R>) -> Self {
        match o {
            PatternObject::Any => Self::Any,
            PatternObject::SameAsSubject => Self::SameAsSubject,
            PatternObject::SameAsPredicate => Self::SameAsPredicate,
            PatternObject::Given(s) => match dataset.get_resource(s) {
                Some(object) => Self::Fixed(sorted_set(&object.as_object)),
                None => Self::None,
            },
        }
    }

    fn new_multi<'p, R, P>(dataset: &IndexedHashDataset<R>, o: PatternObject<P>) -> Self
    where
        P: IntoIterator<Item = &'p R>,
        R: 'p + Eq + Hash,
    {
        match o {
            PatternObject::Any => Self::Any,
            PatternObject::SameAsSubject => Self::SameAsSubject,
            PatternObject::SameAsPredicate => Self::SameAsPredicate,
            PatternObject::Given(multi_o) => {
                let mut indexes = Vec::new();
                for o in multi_o {
                    if let Some(object) = dataset.get_resource(o) {
                        indexes.extend(object.as_object.iter().copied());
                    }
                }

                if indexes.is_empty() {
                    Self::None
                } else {
                    indexes.sort_unstable();
                    Self::Fixed(indexes.into_iter().peekable())
                }
            }
        }
    }

    fn next(&mut self, i: usize, quad: Quad<usize>) -> Result<(), Option<usize>> {
        match self {
            Self::None => Err(None),
            Self::Any => Ok(()),
            Self::SameAsSubject => {
                if quad.0 == quad.2 {
                    Ok(())
                } else {
                    Err(i.checked_add(1))
                }
            }
            Self::SameAsPredicate => {
                if quad.1 == quad.2 {
                    Ok(())
                } else {
                    Err(i.checked_add(1))
                }
            }
            Self::Fixed(indexes) => {
                while let Some(j) = indexes.peek().copied() {
                    match j.cmp(&i) {
                        Ordering::Equal => return Ok(()),
                        Ordering::Greater => return Err(Some(j)),
                        Ordering::Less => {
                            indexes.next();
                        }
                    }
                }

                Err(None)
            }
        }
    }
}

enum GraphConstraints {
    None,
    Any,
    SameAsSubject,
    SameAsPredicate,
    SameAsObject,
    Fixed(SortedIndexes),
}

impl GraphConstraints {
    fn new<R: Eq + Hash>(dataset: &IndexedHashDataset<R>, g: PatternGraph<&R>) -> Self {
        match g {
            PatternGraph::Any => Self::Any,
            PatternGraph::SameAsSubject => Self::SameAsSubject,
            PatternGraph::SameAsPredicate => Self::SameAsPredicate,
            PatternGraph::SameAsObject => Self::SameAsObject,
            PatternGraph::Given(Some(s)) => match dataset.get_resource(s) {
                Some(g) => Self::Fixed(sorted_set(&g.as_graph)),
                None => Self::None,
            },
            PatternGraph::Given(None) => Self::Fixed(sorted_set(&dataset.default_graph)),
        }
    }

    fn new_multi<'p, R, P>(dataset: &IndexedHashDataset<R>, g: PatternGraph<P>) -> Self
    where
        P: IntoIterator<Item = &'p R>,
        R: 'p + Eq + Hash,
    {
        match g {
            PatternGraph::Any => Self::Any,
            PatternGraph::SameAsSubject => Self::SameAsSubject,
            PatternGraph::SameAsPredicate => Self::SameAsPredicate,
            PatternGraph::SameAsObject => Self::SameAsObject,
            PatternGraph::Given(Some(multi_g)) => {
                let mut indexes = Vec::new();
                for g in multi_g {
                    if let Some(graph) = dataset.get_resource(g) {
                        indexes.extend(graph.as_graph.iter().copied());
                    }
                }

                if indexes.is_empty() {
                    Self::None
                } else {
                    indexes.sort_unstable();
                    Self::Fixed(indexes.into_iter().peekable())
                }
            }
            PatternGraph::Given(None) => Self::Fixed(sorted_set(&dataset.default_graph)),
        }
    }

    fn next(&mut self, i: usize, quad: Quad<usize>) -> Result<(), Option<usize>> {
        match self {
            Self::None => Err(None),
            Self::Any => Ok(()),
            Self::SameAsSubject => {
                if Some(quad.0) == quad.3 {
                    Ok(())
                } else {
                    Err(i.checked_add(1))
                }
            }
            Self::SameAsPredicate => {
                if Some(quad.1) == quad.3 {
                    Ok(())
                } else {
                    Err(i.checked_add(1))
                }
            }
            Self::SameAsObject => {
                if Some(quad.2) == quad.3 {
                    Ok(())
                } else {
                    Err(i.checked_add(1))
                }
            }
            Self::Fixed(indexes) => {
                while let Some(j) = indexes.peek().copied() {
                    match j.cmp(&i) {
                        Ordering::Equal => return Ok(()),
                        Ordering::Greater => return Err(Some(j)),
                        Ordering::Less => {
                            indexes.next();
                        }
                    }
                }

                Err(None)
            }
        }
    }
}

#[derive(Default, Clone)]
struct Resource<R> {
    value: R,
    as_subject: BTreeSet<usize>,
    as_predicate: BTreeSet<usize>,
    as_object: BTreeSet<usize>,
    as_graph: BTreeSet<usize>,
}

impl<R> Resource<R> {
    pub fn subject(value: R, i: usize) -> Self {
        let mut s = BTreeSet::new();
        s.insert(i);
        Self {
            value,
            as_subject: s,
            as_predicate: BTreeSet::new(),
            as_object: BTreeSet::new(),
            as_graph: BTreeSet::new(),
        }
    }

    pub fn predicate(value: R, i: usize) -> Self {
        let mut s = BTreeSet::new();
        s.insert(i);
        Self {
            value,
            as_subject: BTreeSet::new(),
            as_predicate: s,
            as_object: BTreeSet::new(),
            as_graph: BTreeSet::new(),
        }
    }

    pub fn object(value: R, i: usize) -> Self {
        let mut s = BTreeSet::new();
        s.insert(i);
        Self {
            value,
            as_subject: BTreeSet::new(),
            as_predicate: BTreeSet::new(),
            as_object: s,
            as_graph: BTreeSet::new(),
        }
    }

    pub fn graph(value: R, i: usize) -> Self {
        let mut s = BTreeSet::new();
        s.insert(i);
        Self {
            value,
            as_subject: BTreeSet::new(),
            as_predicate: BTreeSet::new(),
            as_object: BTreeSet::new(),
            as_graph: s,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.as_subject.is_empty() && self.as_predicate.is_empty() && self.as_object.is_empty() && self.as_graph.is_empty()
    }
}

impl<R: Debug> Debug for IndexedHashDataset<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<R: RdfDisplay> RdfDisplay for IndexedHashDataset<R> {
    fn rdf_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for t in self {
            writeln!(f, "{} .", t.rdf_display())?;
        }

        Ok(())
    }
}

#[cfg(feature = "serde")]
impl<R: serde::Serialize> serde::Serialize for IndexedHashDataset<R> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for quad in self {
            seq.serialize_element(&quad)?;
        }

        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, R: Clone + Eq + Hash + serde::Deserialize<'de>> serde::Deserialize<'de> for IndexedHashDataset<R> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<R>(std::marker::PhantomData<R>);

        impl<'de, R: Clone + Eq + Hash + serde::Deserialize<'de>> serde::de::Visitor<'de> for Visitor<R> {
            type Value = IndexedHashDataset<R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an RDF dataset")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut result = IndexedHashDataset::new();

                while let Some(quad) = seq.next_element()? {
                    result.insert(quad);
                }

                Ok(result)
            }
        }

        deserializer.deserialize_seq(Visitor(std::marker::PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng, rngs::SmallRng};

    use crate::Quad;

    use super::IndexedHashDataset;

    fn rng_graph(rng: &mut SmallRng) -> Option<u32> {
        let g = rng.next_u32();
        if g % 2 == 0 { Some(g) } else { None }
    }

    fn insert_test(n: usize, seed: [u8; 32]) {
        let mut rng = SmallRng::from_seed(seed);
        let mut quads = Vec::new();
        quads.resize_with(n, || Quad(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng_graph(&mut rng)));

        let mut dataset = IndexedHashDataset::new();
        for &t in &quads {
            dataset.insert(t);
        }

        quads.sort_unstable();
        quads.dedup();

        assert_eq!(dataset.len(), quads.len());
        for q in &quads {
            assert!(dataset.contains(q.as_ref()));
        }
    }

    fn remove_test(n: usize, seed: [u8; 32]) {
        use rand::prelude::SliceRandom;
        let mut rng = SmallRng::from_seed(seed);
        let mut quads = Vec::new();
        quads.resize_with(n, || Quad(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng_graph(&mut rng)));

        let mut dataset = IndexedHashDataset::new();
        for &t in &quads {
            dataset.insert(t);
        }

        quads.shuffle(&mut rng);

        for _ in 0..(n / 2) {
            let t = quads.pop().unwrap();
            dataset.remove(t.as_ref());
        }

        quads.sort_unstable();
        quads.dedup();

        assert_eq!(dataset.len(), quads.len());
        for q in &quads {
            assert!(dataset.contains(q.as_ref()));
        }
    }

    #[test]
    fn insert() {
        for i in 0u8..32 {
            insert_test(i as usize * 11, [i; 32]);
        }
    }

    #[test]
    fn remove() {
        for i in 0u8..32 {
            remove_test(i as usize * 11, [i; 32]);
        }
    }
}
