use std::{
    cmp::Ordering,
    collections::BTreeSet,
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

use hashbrown::{DefaultHashBuilder, HashSet, HashTable};
use slab::Slab;

use super::{super::Graph, hash_graph::HashGraph};
use crate::{
    BlankIdBuf,
    LocalTerm,
    RdfDisplay,
    Triple,
    dataset::{
        GraphMut,
        ObjectTraversableGraph,
        PatternMatchingGraph,
        PredicateTraversableGraph,
        ResourceTraversableGraph,
        SubjectTraversableGraph,
        TraversableGraph,
        isomorphism::{Bijection, are_isomorphic, find_bijection},
    },
    pattern::{
        CanonicalTriplePattern,
        triple::canonical::{PatternObject, PatternPredicate, PatternSubject},
    },
};

fn triple_with_resources<R>(resources: &Slab<Resource<R>>, Triple(s, p, o): Triple<usize>) -> Triple<&R> {
    Triple(&resources[s].value, &resources[p].value, &resources[o].value)
}

/// Indexed hash-based RDF graph, optimized for pattern matching operations.
#[derive(Clone)]
pub struct IndexedHashGraph<R = LocalTerm> {
    resources: Slab<Resource<R>>,
    triples: Slab<Triple<usize>>,
    hasher: DefaultHashBuilder,
    resources_indexes: HashTable<usize>,
    triples_indexes: HashTable<usize>,
    subjects: HashSet<usize>,
    predicates: HashSet<usize>,
    objects: HashSet<usize>,
}

impl<R> Default for IndexedHashGraph<R> {
    fn default() -> Self {
        Self {
            triples: Slab::new(),
            resources: Slab::new(),
            hasher: DefaultHashBuilder::default(),
            triples_indexes: HashTable::new(),
            resources_indexes: HashTable::new(),
            subjects: HashSet::default(),
            predicates: HashSet::default(),
            objects: HashSet::default(),
        }
    }
}

impl<R> IndexedHashGraph<R> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.triples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }

    pub fn iter(&self) -> Triples<'_, R> {
        Triples {
            resources: &self.resources,
            triples: &self.triples,
            indexes: self.triples_indexes.iter(),
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
}

impl<R: Eq + Hash> IndexedHashGraph<R> {
    pub fn from_non_indexed(graph: HashGraph<R>) -> Self {
        let mut resources: Slab<Resource<R>> = graph
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
                    },
                )
            })
            .collect();

        let mut subjects = HashSet::default();
        let mut predicates = HashSet::default();
        let mut objects = HashSet::default();

        for &i in &graph.triples_indexes {
            let Triple(s, p, o) = graph.triples[i];

            resources[s].as_subject.insert(i);
            subjects.insert(s);

            resources[p].as_predicate.insert(i);
            predicates.insert(p);

            resources[o].as_object.insert(i);
            objects.insert(o);
        }

        Self {
            resources,
            triples: graph.triples,
            hasher: graph.hasher,
            resources_indexes: graph.resources_indexes,
            triples_indexes: graph.triples_indexes,
            subjects,
            predicates,
            objects,
        }
    }

    fn hash_resource(&self, r: &R) -> u64 {
        self.hasher.hash_one(r)
    }

    fn hash_triple(&self, t: &Triple<&R>) -> u64 {
        self.hasher.hash_one(t)
    }

    fn index_of_resource(&self, r: &R) -> Option<usize> {
        let h = self.hash_resource(r);
        let resources = &self.resources;
        self.resources_indexes.find(h, |&i| &resources[i].value == r).copied()
    }

    fn get_resource(&self, r: &R) -> Option<&Resource<R>> {
        self.resources.get(self.index_of_resource(r)?)
    }

    fn index_of_triple(&self, triple: Triple<&R>) -> Option<usize> {
        let h = self.hash_triple(&triple);
        let resources = &self.resources;
        let triples = &self.triples;
        self.triples_indexes.find(h, |&i| triple_with_resources(resources, triples[i]) == triple).copied()
    }

    pub fn contains_resource(&self, r: &R) -> bool {
        self.index_of_resource(r).is_some()
    }

    pub fn contains(&self, triple: Triple<&R>) -> bool {
        self.index_of_triple(triple).is_some()
    }
}

impl<R: Clone + Eq + Hash> IndexedHashGraph<R> {
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

    pub fn insert(&mut self, triple: Triple<R>) -> bool {
        if self.contains(triple.as_ref()) {
            return false;
        }

        let s_pre = self.index_of_resource(&triple.0);
        let p_pre = self.index_of_resource(&triple.1);
        let o_pre = self.index_of_resource(&triple.2);

        let i = self.triples.vacant_key();

        let s_i = self.intern_resource(triple.0, s_pre, Resource::subject, i);
        if s_pre.is_some() {
            self.resources[s_i].as_subject.insert(i);
        }

        let p_i = self.intern_resource(triple.1, p_pre, Resource::predicate, i);
        if p_pre.is_some() {
            self.resources[p_i].as_predicate.insert(i);
        }

        let o_i = self.intern_resource(triple.2, o_pre, Resource::object, i);
        if o_pre.is_some() {
            self.resources[o_i].as_object.insert(i);
        }

        self.subjects.insert(s_i);
        self.predicates.insert(p_i);
        self.objects.insert(o_i);

        let stored = Triple(s_i, p_i, o_i);
        let inserted = self.triples.insert(stored);
        debug_assert_eq!(inserted, i);

        let h = self.hasher.hash_one(&triple_with_resources(&self.resources, stored));
        let hasher = self.hasher.clone();
        let resources = &self.resources;
        let triples = &self.triples;
        self.triples_indexes
            .insert_unique(h, i, |&j| hasher.hash_one(&triple_with_resources(resources, triples[j])));

        true
    }

    fn drop_resource_if_empty(&mut self, ri: usize) {
        if self.resources[ri].is_empty() {
            let h = self.hash_resource(&self.resources[ri].value);
            if let Ok(entry) = self.resources_indexes.find_entry(h, |&j| j == ri) {
                entry.remove();
            }
            self.resources.remove(ri);
        }
    }

    pub fn remove(&mut self, triple: Triple<&R>) -> bool {
        let h = self.hash_triple(&triple);
        let resources = &self.resources;
        let triples = &self.triples;
        let entry = self
            .triples_indexes
            .find_entry(h, |&i| triple_with_resources(resources, triples[i]) == triple);

        let i = match entry {
            Ok(e) => {
                let (i, _) = e.remove();
                i
            }
            Err(_) => return false,
        };

        let Triple(s_i, p_i, o_i) = self.triples.remove(i);

        self.subjects.remove(&s_i);
        self.predicates.remove(&p_i);
        self.objects.remove(&o_i);

        self.resources[s_i].as_subject.remove(&i);
        self.drop_resource_if_empty(s_i);
        self.resources[p_i].as_predicate.remove(&i);
        self.drop_resource_if_empty(p_i);
        self.resources[o_i].as_object.remove(&i);
        self.drop_resource_if_empty(o_i);

        true
    }

}

impl<R: Eq + Hash> IndexedHashGraph<R> {
    pub fn pattern_matching(&self, pattern: CanonicalTriplePattern<&R>) -> PatternMatching<'_, R> {
        PatternMatching {
            resources: &self.resources,
            triples: &self.triples,
            subject: SubjectConstraints::new(self, pattern.into_subject()),
            predicate: PredicateConstraints::new(self, pattern.into_predicate()),
            object: ObjectConstraints::new(self, pattern.into_object()),
            i: 0,
        }
    }
}

impl<R: Clone + Eq + Hash> From<HashGraph<R>> for IndexedHashGraph<R> {
    fn from(value: HashGraph<R>) -> Self {
        Self::from_non_indexed(value)
    }
}

impl<R: Clone + Eq + Hash> FromIterator<Triple<R>> for IndexedHashGraph<R> {
    fn from_iter<T: IntoIterator<Item = Triple<R>>>(iter: T) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl<R: Clone + Eq + Hash> Extend<Triple<R>> for IndexedHashGraph<R> {
    fn extend<T: IntoIterator<Item = Triple<R>>>(&mut self, iter: T) {
        for triple in iter {
            self.insert(triple);
        }
    }
}

impl<R: Clone + Eq + Hash> IndexedHashGraph<R> {
    /// Inserts every triple of `iter` in the graph.
    pub fn absorb<I: IntoIterator<Item = Triple<R>>>(&mut self, iter: I) {
        self.extend(iter);
    }

    /// Removes the given triple and returns it if it was present.
    pub fn take(&mut self, triple: Triple<&R>) -> Option<Triple<R>> {
        if !self.contains(triple) {
            return None;
        }
        let value = triple.cloned();
        self.remove(triple);
        Some(value)
    }

    /// Removes and returns the first triple matching the given option-pattern, if any.
    pub fn take_match(&mut self, pattern: Triple<Option<&R>, Option<&R>, Option<&R>>) -> Option<Triple<R>> {
        let canonical = CanonicalTriplePattern::from_option_triple(pattern);
        let triple: Triple<R> = self.pattern_matching(canonical).next()?.cloned();
        self.remove(triple.as_ref());
        Some(triple)
    }
}

impl IndexedHashGraph<LocalTerm> {
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
            .map(|t| {
                t.map(|x| match x {
                    LocalTerm::Anonymous(b) => LocalTerm::Anonymous(f(b)),
                    named => named,
                })
            })
            .collect::<HashGraph<LocalTerm>>()
            .into_indexed()
    }
}

impl<R> Graph for IndexedHashGraph<R> {
    type Resource = R;
}

impl<R> TraversableGraph for IndexedHashGraph<R> {
    type Triples<'a>
        = Triples<'a, R>
    where
        R: 'a;

    fn triples(&self) -> Self::Triples<'_> {
        self.iter()
    }
}

impl<R> ResourceTraversableGraph for IndexedHashGraph<R> {
    type GraphResources<'a>
        = Resources<'a, R>
    where
        R: 'a;

    fn graph_resources(&self) -> Self::GraphResources<'_> {
        self.resources()
    }

    fn graph_resource_count(&self) -> usize {
        self.resources.len()
    }
}

impl<R> SubjectTraversableGraph for IndexedHashGraph<R> {
    type GraphSubjects<'a>
        = Subjects<'a, R>
    where
        R: 'a;

    fn graph_subjects(&self) -> Self::GraphSubjects<'_> {
        self.subjects()
    }

    fn graph_subject_count(&self) -> usize {
        self.subjects.len()
    }
}

impl<R> PredicateTraversableGraph for IndexedHashGraph<R> {
    type GraphPredicates<'a>
        = Predicates<'a, R>
    where
        R: 'a;

    fn graph_predicates(&self) -> Self::GraphPredicates<'_> {
        self.predicates()
    }

    fn graph_predicate_count(&self) -> usize {
        self.predicates.len()
    }
}

impl<R> ObjectTraversableGraph for IndexedHashGraph<R> {
    type GraphObjects<'a>
        = Objects<'a, R>
    where
        R: 'a;

    fn graph_objects(&self) -> Self::GraphObjects<'_> {
        self.objects()
    }

    fn graph_object_count(&self) -> usize {
        self.objects.len()
    }
}

impl<R: Clone + Eq + Hash> GraphMut for IndexedHashGraph<R> {
    fn insert(&mut self, triple: Triple<Self::Resource>) {
        self.insert(triple);
    }

    fn remove(&mut self, triple: Triple<&Self::Resource>) {
        self.remove(triple);
    }
}

impl<R: Eq + Hash> PatternMatchingGraph for IndexedHashGraph<R> {
    type TriplePatternMatching<'a, 'p>
        = PatternMatching<'a, R>
    where
        R: 'a,
        Self::Resource: 'p;

    fn triple_pattern_matching<'p>(&self, pattern: CanonicalTriplePattern<&'p Self::Resource>) -> Self::TriplePatternMatching<'_, 'p> {
        self.pattern_matching(pattern)
    }

    fn contains_triple(&self, triple: Triple<&Self::Resource>) -> bool {
        self.contains(triple)
    }
}

pub struct Triples<'a, R> {
    resources: &'a Slab<Resource<R>>,
    triples: &'a Slab<Triple<usize>>,
    indexes: hashbrown::hash_table::Iter<'a, usize>,
}

impl<'a, R> Iterator for Triples<'a, R> {
    type Item = Triple<&'a R>;

    fn next(&mut self) -> Option<Self::Item> {
        self.indexes.next().map(|&i| triple_with_resources(self.resources, self.triples[i]))
    }
}

pub struct IntoTriples<R> {
    resources: Slab<Resource<R>>,
    triples: Slab<Triple<usize>>,
    indexes: hashbrown::hash_table::IntoIter<usize>,
}

impl<R: Clone> Iterator for IntoTriples<R> {
    type Item = Triple<R>;

    fn next(&mut self) -> Option<Self::Item> {
        self.indexes
            .next()
            .map(|i| triple_with_resources(&self.resources, self.triples.remove(i)).cloned())
    }
}

impl<'a, R> IntoIterator for &'a IndexedHashGraph<R> {
    type Item = Triple<&'a R>;
    type IntoIter = Triples<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<R: Clone> IntoIterator for IndexedHashGraph<R> {
    type Item = Triple<R>;
    type IntoIter = IntoTriples<R>;

    fn into_iter(self) -> Self::IntoIter {
        IntoTriples {
            resources: self.resources,
            triples: self.triples,
            indexes: self.triples_indexes.into_iter(),
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

impl<R: Eq + Hash> PartialEq for IndexedHashGraph<R> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().all(|t| other.contains(t))
    }
}

impl<R: Eq + Hash> Eq for IndexedHashGraph<R> {}

pub struct PatternMatching<'a, R> {
    resources: &'a Slab<Resource<R>>,
    triples: &'a Slab<Triple<usize>>,
    subject: SubjectConstraints,
    predicate: PredicateConstraints,
    object: ObjectConstraints,
    i: usize,
}

impl<'a, R> Iterator for PatternMatching<'a, R> {
    type Item = Triple<&'a R>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.triples.capacity() {
            let i = self.subject.next(self.i)?;
            let triple = *self.triples.get(i)?;
            match self.predicate.next(i, triple) {
                Ok(()) => match self.object.next(i, triple) {
                    Ok(()) => {
                        self.i = i + 1;
                        return Some(triple_with_resources(self.resources, triple));
                    }
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
    fn new<R: Eq + Hash>(graph: &IndexedHashGraph<R>, s: PatternSubject<&R>) -> Self {
        match s {
            PatternSubject::Any => Self::Any,
            PatternSubject::Given(s) => match graph.get_resource(s) {
                Some(subject) => Self::Fixed(sorted_set(&subject.as_subject)),
                None => Self::None,
            },
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
    fn new<R: Eq + Hash>(graph: &IndexedHashGraph<R>, p: PatternPredicate<&R>) -> Self {
        match p {
            PatternPredicate::Any => Self::Any,
            PatternPredicate::SameAsSubject => Self::SameAsSubject,
            PatternPredicate::Given(s) => match graph.get_resource(s) {
                Some(predicate) => Self::Fixed(sorted_set(&predicate.as_predicate)),
                None => Self::None,
            },
        }
    }

    fn next(&mut self, i: usize, triple: Triple<usize>) -> Result<(), Option<usize>> {
        match self {
            Self::None => Err(None),
            Self::Any => Ok(()),
            Self::SameAsSubject => {
                if triple.0 == triple.1 {
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
    fn new<R: Eq + Hash>(graph: &IndexedHashGraph<R>, o: PatternObject<&R>) -> Self {
        match o {
            PatternObject::Any => Self::Any,
            PatternObject::SameAsSubject => Self::SameAsSubject,
            PatternObject::SameAsPredicate => Self::SameAsPredicate,
            PatternObject::Given(s) => match graph.get_resource(s) {
                Some(object) => Self::Fixed(sorted_set(&object.as_object)),
                None => Self::None,
            },
        }
    }

    fn next(&mut self, i: usize, triple: Triple<usize>) -> Result<(), Option<usize>> {
        match self {
            Self::None => Err(None),
            Self::Any => Ok(()),
            Self::SameAsSubject => {
                if triple.0 == triple.2 {
                    Ok(())
                } else {
                    Err(i.checked_add(1))
                }
            }
            Self::SameAsPredicate => {
                if triple.1 == triple.2 {
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
pub(crate) struct Resource<R> {
    pub value: R,
    pub as_subject: BTreeSet<usize>,
    pub as_predicate: BTreeSet<usize>,
    pub as_object: BTreeSet<usize>,
}

impl<R> Resource<R> {
    pub fn subject(value: R, i: usize) -> Self {
        let mut as_subject = BTreeSet::new();
        as_subject.insert(i);
        Self {
            value,
            as_subject,
            as_predicate: BTreeSet::new(),
            as_object: BTreeSet::new(),
        }
    }

    pub fn predicate(value: R, i: usize) -> Self {
        let mut as_predicate = BTreeSet::new();
        as_predicate.insert(i);
        Self {
            value,
            as_subject: BTreeSet::new(),
            as_predicate,
            as_object: BTreeSet::new(),
        }
    }

    pub fn object(value: R, i: usize) -> Self {
        let mut as_object = BTreeSet::new();
        as_object.insert(i);
        Self {
            value,
            as_subject: BTreeSet::new(),
            as_predicate: BTreeSet::new(),
            as_object,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.as_subject.is_empty() && self.as_predicate.is_empty() && self.as_object.is_empty()
    }
}

impl<R: Debug> Debug for IndexedHashGraph<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<R: RdfDisplay> RdfDisplay for IndexedHashGraph<R> {
    fn rdf_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for t in self {
            writeln!(f, "{} .", t.rdf_display())?;
        }

        Ok(())
    }
}

#[cfg(feature = "serde")]
impl<R: serde::Serialize> serde::Serialize for IndexedHashGraph<R> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for triple in self {
            seq.serialize_element(&triple)?;
        }

        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, R: Clone + Eq + Hash + serde::Deserialize<'de>> serde::Deserialize<'de> for IndexedHashGraph<R> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<R>(std::marker::PhantomData<R>);

        impl<'de, R: Clone + Eq + Hash + serde::Deserialize<'de>> serde::de::Visitor<'de> for Visitor<R> {
            type Value = IndexedHashGraph<R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an RDF graph")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut result = IndexedHashGraph::new();

                while let Some(triple) = seq.next_element()? {
                    result.insert(triple);
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

    use crate::Triple;

    use super::IndexedHashGraph;

    fn insert_test(n: usize, seed: [u8; 32]) {
        let mut rng = SmallRng::from_seed(seed);
        let mut triples = Vec::new();
        triples.resize_with(n, || Triple(rng.next_u32(), rng.next_u32(), rng.next_u32()));

        let mut graph = IndexedHashGraph::new();
        for &t in &triples {
            graph.insert(t);
        }

        triples.sort_unstable();
        triples.dedup();

        assert_eq!(graph.len(), triples.len());
        for t in &triples {
            assert!(graph.contains(t.as_ref()));
        }
    }

    fn remove_test(n: usize, seed: [u8; 32]) {
        use rand::prelude::SliceRandom;
        let mut rng = SmallRng::from_seed(seed);
        let mut triples = Vec::new();
        triples.resize_with(n, || Triple(rng.next_u32(), rng.next_u32(), rng.next_u32()));

        let mut graph = IndexedHashGraph::new();
        for &t in &triples {
            graph.insert(t);
        }

        triples.shuffle(&mut rng);

        for _ in 0..(n / 2) {
            let t = triples.pop().unwrap();
            graph.remove(t.as_ref());
        }

        triples.sort_unstable();
        triples.dedup();

        assert_eq!(graph.len(), triples.len());
        for t in &triples {
            assert!(graph.contains(t.as_ref()));
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
