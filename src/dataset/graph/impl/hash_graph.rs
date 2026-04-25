use std::{
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

use hashbrown::{DefaultHashBuilder, HashTable};
use slab::Slab;

use crate::{
    BlankIdBuf,
    LocalTerm,
    RdfDisplay,
    Triple,
    dataset::{
        Graph,
        GraphMut,
        IndexedHashGraph,
        ResourceTraversableGraph,
        TraversableGraph,
        isomorphism::{Bijection, are_isomorphic, find_bijection},
    },
};

fn triple_with_resources<R>(resources: &Slab<Resource<R>>, Triple(s, p, o): Triple<usize>) -> Triple<&R> {
    Triple(&resources[s].value, &resources[p].value, &resources[o].value)
}

/// Hash-based RDF graph.
#[derive(Clone)]
pub struct HashGraph<R = LocalTerm> {
    pub(crate) resources: Slab<Resource<R>>,
    pub(crate) triples: Slab<Triple<usize>>,
    pub(crate) hasher: DefaultHashBuilder,
    pub(crate) resources_indexes: HashTable<usize>,
    pub(crate) triples_indexes: HashTable<usize>,
}

impl<R> Default for HashGraph<R> {
    fn default() -> Self {
        Self {
            triples: Slab::new(),
            resources: Slab::new(),
            hasher: DefaultHashBuilder::default(),
            triples_indexes: HashTable::new(),
            resources_indexes: HashTable::new(),
        }
    }
}

impl<R> HashGraph<R> {
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
}

impl<R: Eq + Hash> HashGraph<R> {
    pub fn into_indexed(self) -> IndexedHashGraph<R> {
        IndexedHashGraph::from_non_indexed(self)
    }
}

impl<R: Eq + Hash> HashGraph<R> {
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

    fn index_of_triple(&self, triple: Triple<&R>) -> Option<usize> {
        let h = self.hash_triple(&triple);
        let resources = &self.resources;
        let triples = &self.triples;
        self.triples_indexes
            .find(h, |&i| triple_with_resources(resources, triples[i]) == triple)
            .copied()
    }

    pub fn contains_resource(&self, resource: &R) -> bool {
        self.index_of_resource(resource).is_some()
    }

    pub fn contains(&self, triple: Triple<&R>) -> bool {
        self.index_of_triple(triple).is_some()
    }
}

impl<R: Clone + Eq + Hash> HashGraph<R> {
    fn intern_resource(&mut self, r: R, pre: Option<usize>) -> usize {
        match pre {
            Some(i) => {
                self.resources[i].occurrences += 1;
                i
            }
            None => {
                let h = self.hasher.hash_one(&r);
                let i = self.resources.insert(Resource::new(r));
                let hasher = self.hasher.clone();
                let resources = &self.resources;
                self.resources_indexes.insert_unique(h, i, |&j| hasher.hash_one(&resources[j].value));
                i
            }
        }
    }

    pub fn insert(&mut self, triple: Triple<R>) -> bool {
        let s_pre = self.index_of_resource(&triple.0);
        let p_pre = self.index_of_resource(&triple.1);
        let o_pre = self.index_of_resource(&triple.2);

        let mut precomputed_hash: Option<u64> = None;
        if let (Some(s_i), Some(p_i), Some(o_i)) = (s_pre, p_pre, o_pre) {
            let target = Triple(s_i, p_i, o_i);
            let resolved = triple_with_resources(&self.resources, target);
            let h = self.hasher.hash_one(&resolved);
            let resources = &self.resources;
            let triples = &self.triples;
            if self
                .triples_indexes
                .find(h, |&i| triple_with_resources(resources, triples[i]) == resolved)
                .is_some()
            {
                return false;
            }
            precomputed_hash = Some(h);
        }

        let s_i = self.intern_resource(triple.0, s_pre);
        let p_i = self.intern_resource(triple.1, p_pre);
        let o_i = self.intern_resource(triple.2, o_pre);

        let stored = Triple(s_i, p_i, o_i);
        let h = match precomputed_hash {
            Some(h) => h,
            None => self.hasher.hash_one(&triple_with_resources(&self.resources, stored)),
        };
        let i = self.triples.insert(stored);

        let hasher = self.hasher.clone();
        let resources = &self.resources;
        let triples = &self.triples;
        self.triples_indexes
            .insert_unique(h, i, |&j| hasher.hash_one(&triple_with_resources(resources, triples[j])));

        true
    }

    fn remove_resource_occurrence(&mut self, i: usize, value: &R) {
        let r = &mut self.resources[i];
        r.occurrences -= 1;
        if r.is_empty() {
            let h = self.hasher.hash_one(value);
            if let Ok(entry) = self.resources_indexes.find_entry(h, |&j| j == i) {
                entry.remove();
            }
            self.resources.remove(i);
        }
    }

    pub fn remove(&mut self, triple: Triple<&R>) -> bool {
        let h = self.hash_triple(&triple);
        let resources = &self.resources;
        let triples = &self.triples;
        let entry = self.triples_indexes.find_entry(h, |&i| triple_with_resources(resources, triples[i]) == triple);

        let i = match entry {
            Ok(e) => {
                let (i, _) = e.remove();
                i
            }
            Err(_) => return false,
        };

        let Triple(s_i, p_i, o_i) = self.triples.remove(i);
        self.remove_resource_occurrence(s_i, triple.0);
        self.remove_resource_occurrence(p_i, triple.1);
        self.remove_resource_occurrence(o_i, triple.2);
        true
    }
}

impl<R: Clone + Eq + Hash> FromIterator<Triple<R>> for HashGraph<R> {
    fn from_iter<T: IntoIterator<Item = Triple<R>>>(iter: T) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl<R: Clone + Eq + Hash> Extend<Triple<R>> for HashGraph<R> {
    fn extend<T: IntoIterator<Item = Triple<R>>>(&mut self, iter: T) {
        for triple in iter {
            self.insert(triple);
        }
    }
}

impl<R: Clone + Eq + Hash> HashGraph<R> {
    /// Inserts every triple of `iter` in the graph.
    pub fn absorb<I: IntoIterator<Item = Triple<R>>>(&mut self, iter: I) {
        self.extend(iter);
    }
}

impl HashGraph<LocalTerm> {
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
            .collect()
    }
}

impl<R> Graph for HashGraph<R> {
    type Resource = R;
}

impl<R> TraversableGraph for HashGraph<R> {
    type Triples<'a>
        = Triples<'a, R>
    where
        R: 'a;

    fn triples(&self) -> Self::Triples<'_> {
        self.iter()
    }

    fn triples_count(&self) -> usize {
        self.len()
    }
}

impl<R> ResourceTraversableGraph for HashGraph<R> {
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

impl<R: Clone + Eq + Hash> GraphMut for HashGraph<R> {
    fn insert(&mut self, triple: Triple<Self::Resource>) {
        self.insert(triple);
    }

    fn remove(&mut self, triple: Triple<&Self::Resource>) {
        self.remove(triple);
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

impl<'a, R> IntoIterator for &'a HashGraph<R> {
    type Item = Triple<&'a R>;
    type IntoIter = Triples<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<R: Clone> IntoIterator for HashGraph<R> {
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

impl<R: Eq + Hash> PartialEq for HashGraph<R> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().all(|t| other.contains(t))
    }
}

impl<R: Eq + Hash> Eq for HashGraph<R> {}

#[derive(Default, Clone)]
pub(crate) struct Resource<R> {
    pub value: R,
    occurrences: usize,
}

impl<R> Resource<R> {
    pub fn new(value: R) -> Self {
        Self { value, occurrences: 1 }
    }

    pub fn is_empty(&self) -> bool {
        self.occurrences == 0
    }
}

impl<R: Debug> Debug for HashGraph<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<R: RdfDisplay> RdfDisplay for HashGraph<R> {
    fn rdf_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for t in self {
            writeln!(f, "{} .", t.rdf_display())?;
        }

        Ok(())
    }
}

#[cfg(feature = "serde")]
impl<R: serde::Serialize> serde::Serialize for HashGraph<R> {
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
impl<'de, R: Clone + Eq + Hash + serde::Deserialize<'de>> serde::Deserialize<'de> for HashGraph<R> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<R>(std::marker::PhantomData<R>);

        impl<'de, R: Clone + Eq + Hash + serde::Deserialize<'de>> serde::de::Visitor<'de> for Visitor<R> {
            type Value = HashGraph<R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an RDF graph")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut result = HashGraph::new();

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

    use super::HashGraph;

    fn insert_test(n: usize, seed: [u8; 32]) {
        let mut rng = SmallRng::from_seed(seed);
        let mut triples = Vec::new();
        triples.resize_with(n, || Triple(rng.next_u32(), rng.next_u32(), rng.next_u32()));

        let mut graph = HashGraph::new();
        for &t in &triples {
            graph.insert(t);
        }

        triples.sort_unstable();
        triples.dedup();

        assert_eq!(graph.len(), triples.len());
    }

    fn remove_test(n: usize, seed: [u8; 32]) {
        use rand::prelude::SliceRandom;
        let mut rng = SmallRng::from_seed(seed);
        let mut triples = Vec::new();
        triples.resize_with(n, || Triple(rng.next_u32(), rng.next_u32(), rng.next_u32()));

        let mut graph = HashGraph::new();
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
