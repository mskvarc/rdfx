use std::{
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
};

use hashbrown::{DefaultHashBuilder, HashTable};
use slab::Slab;

use super::{super::Dataset, btree_dataset::Resource};
use crate::{
    BlankIdBuf,
    LocalTerm,
    Quad,
    RdfDisplay,
    dataset::{
        DatasetMut,
        IndexedHashDataset,
        ResourceTraversableDataset,
        TraversableDataset,
        isomorphism::{Bijection, are_isomorphic, find_bijection},
    },
};

fn quad_with_resources<R>(resources: &Slab<Resource<R>>, Quad(s, p, o, g): Quad<usize>) -> Quad<&R> {
    // Safety: resource indexes in stored quads always point to live entries in `resources`.
    // The slab key remains valid until the quad referencing it is removed, which only happens
    // before the corresponding resource entry is freed.
    unsafe {
        Quad(
            &resources.get_unchecked(s).value,
            &resources.get_unchecked(p).value,
            &resources.get_unchecked(o).value,
            g.map(|g| &resources.get_unchecked(g).value),
        )
    }
}

/// Hash-based RDF dataset.
#[derive(Clone)]
/// Hash-based RDF dataset.
pub struct HashDataset<R = LocalTerm> {
    pub(crate) resources: Slab<Resource<R>>,
    pub(crate) quads: Slab<Quad<usize>>,
    pub(crate) hasher: DefaultHashBuilder,
    pub(crate) resources_indexes: HashTable<usize>,
    pub(crate) quads_indexes: HashTable<usize>,
}

impl<R> Default for HashDataset<R> {
    fn default() -> Self {
        Self {
            quads: Slab::new(),
            resources: Slab::new(),
            hasher: DefaultHashBuilder::default(),
            quads_indexes: HashTable::new(),
            resources_indexes: HashTable::new(),
        }
    }
}

impl<R> HashDataset<R> {
    /// Creates a new empty dataset.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of quads in the dataset.
    pub fn len(&self) -> usize {
        self.quads.len()
    }

    /// Checks if the dataset is empty.
    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
    }

    /// Returns an iterator over the quads of the dataset.
    pub fn iter(&self) -> Quads<'_, R> {
        Quads {
            resources: &self.resources,
            quads: &self.quads,
            indexes: self.quads_indexes.iter(),
        }
    }

    /// Returns an iterator over the resources of the dataset.
    pub fn resources(&self) -> Resources<'_, R> {
        Resources {
            resources: &self.resources,
            indexes: self.resources_indexes.iter(),
        }
    }
}

impl<R: Eq + Hash> HashDataset<R> {
    /// Indexes the quads to allow more operation on this dataset, such as
    /// pattern matching using the [`PatternMatchingDataset`] trait.
    ///
    /// [`PatternMatchingDataset`]: super::super::PatternMatchingDataset
    pub fn into_indexed(self) -> IndexedHashDataset<R> {
        IndexedHashDataset::from_non_indexed(self)
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

    fn index_of_quad(&self, quad: Quad<&R>) -> Option<usize> {
        let h = self.hash_quad(&quad);
        let resources = &self.resources;
        let quads = &self.quads;
        self.quads_indexes.find(h, |&i| quad_with_resources(resources, quads[i]) == quad).copied()
    }

    /// Checks if the provided resource appears in any quad in the dataset.
    pub fn contains_resource(&self, r: &R) -> bool {
        self.index_of_resource(r).is_some()
    }

    /// Checks if the provided quad is in the dataset.
    pub fn contains(&self, q: Quad<&R>) -> bool {
        self.index_of_quad(q).is_some()
    }
}

impl<R: Clone + Eq + Hash> HashDataset<R> {
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

    /// Inserts the given quad in the dataset.
    ///
    /// Returns `true` if the quad was not already in the dataset, and `false`
    /// if it was.
    pub fn insert(&mut self, quad: Quad<R>) -> bool {
        let s_pre = self.index_of_resource(&quad.0);
        let p_pre = self.index_of_resource(&quad.1);
        let o_pre = self.index_of_resource(&quad.2);
        let g_pre = quad.3.as_ref().map(|g| self.index_of_resource(g));

        let g_existing = match g_pre {
            Some(Some(g_i)) => Some(Some(g_i)),
            None => Some(None),
            Some(None) => None,
        };

        let mut precomputed_hash: Option<u64> = None;
        if let (Some(s_i), Some(p_i), Some(o_i), Some(g_slot)) = (s_pre, p_pre, o_pre, g_existing) {
            let target = Quad(s_i, p_i, o_i, g_slot);
            let resolved = quad_with_resources(&self.resources, target);
            let h = self.hasher.hash_one(&resolved);
            let resources = &self.resources;
            let quads = &self.quads;
            if self.quads_indexes.find(h, |&i| quad_with_resources(resources, quads[i]) == resolved).is_some() {
                return false;
            }
            precomputed_hash = Some(h);
        }

        let s_i = self.intern_resource(quad.0, s_pre);
        let p_i = self.intern_resource(quad.1, p_pre);
        let o_i = self.intern_resource(quad.2, o_pre);
        let g_i = match (quad.3, g_pre) {
            (Some(g), Some(pre)) => Some(self.intern_resource(g, pre)),
            (None, None) => None,
            _ => unreachable!(),
        };

        let stored = Quad(s_i, p_i, o_i, g_i);
        let h = match precomputed_hash {
            Some(h) => h,
            None => self.hasher.hash_one(&quad_with_resources(&self.resources, stored)),
        };
        let i = self.quads.insert(stored);

        let hasher = self.hasher.clone();
        let resources = &self.resources;
        let quads = &self.quads;
        self.quads_indexes
            .insert_unique(h, i, |&j| hasher.hash_one(&quad_with_resources(resources, quads[j])));

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

    /// Removes the given quad from the dataset.
    ///
    /// Returns whether or not the quad was in the dataset.
    /// Does nothing if the quad was not in the dataset.
    pub fn remove(&mut self, quad: Quad<&R>) -> bool {
        let h = self.hash_quad(&quad);
        let resources = &self.resources;
        let quads = &self.quads;
        let entry = self.quads_indexes.find_entry(h, |&i| quad_with_resources(resources, quads[i]) == quad);

        let i = match entry {
            Ok(e) => {
                let (i, _) = e.remove();
                i
            }
            Err(_) => return false,
        };

        let Quad(s_i, p_i, o_i, g_i) = self.quads.remove(i);
        self.remove_resource_occurrence(s_i, quad.0);
        self.remove_resource_occurrence(p_i, quad.1);
        self.remove_resource_occurrence(o_i, quad.2);
        if let (Some(g_i), Some(g)) = (g_i, quad.3) {
            self.remove_resource_occurrence(g_i, g);
        }
        true
    }
}

impl<R: Clone + Eq + Hash> FromIterator<Quad<R>> for HashDataset<R> {
    fn from_iter<T: IntoIterator<Item = Quad<R>>>(iter: T) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl<R: Clone + Eq + Hash> Extend<Quad<R>> for HashDataset<R> {
    fn extend<T: IntoIterator<Item = Quad<R>>>(&mut self, iter: T) {
        for quad in iter {
            self.insert(quad);
        }
    }
}

impl<R: Clone + Eq + Hash> HashDataset<R> {
    /// Inserts every quad of `iter` in the dataset.
    pub fn absorb<I: IntoIterator<Item = Quad<R>>>(&mut self, iter: I) {
        self.extend(iter);
    }
}

impl HashDataset<LocalTerm> {
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
                    LocalTerm::BlankId(b) => LocalTerm::BlankId(f(b)),
                    named => named,
                })
            })
            .collect()
    }
}

impl<R> Dataset for HashDataset<R> {
    type Resource = R;
}

impl<R> TraversableDataset for HashDataset<R> {
    type Quads<'a>
        = Quads<'a, R>
    where
        R: 'a;

    fn quads(&self) -> Self::Quads<'_> {
        self.iter()
    }

    fn quads_count(&self) -> usize {
        self.len()
    }
}

impl<R> ResourceTraversableDataset for HashDataset<R> {
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

impl<R: Clone + Eq + Hash> DatasetMut for HashDataset<R> {
    fn insert(&mut self, quad: Quad<Self::Resource>) {
        self.insert(quad);
    }

    fn remove(&mut self, quad: Quad<&Self::Resource>) {
        self.remove(quad);
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

impl<'a, R> IntoIterator for &'a HashDataset<R> {
    type Item = Quad<&'a R>;
    type IntoIter = Quads<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<R: Clone> IntoIterator for HashDataset<R> {
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

impl<A: PartialEq<B>, B> PartialEq<HashDataset<B>> for HashDataset<A> {
    fn eq(&self, other: &HashDataset<B>) -> bool {
        self.len() == other.len() && self.iter().all(|a| other.iter().any(|b| a == b))
    }
}

impl<R: Eq + Hash> Eq for HashDataset<R> {}

impl<R: Hash> Hash for HashDataset<R> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.len());
        let mut acc: u64 = 0;
        for elt in self {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            elt.hash(&mut h);
            acc ^= h.finish();
        }
        state.write_u64(acc);
    }
}

impl<R: Debug> Debug for HashDataset<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<R: RdfDisplay> RdfDisplay for HashDataset<R> {
    fn rdf_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for t in self {
            writeln!(f, "{} .", t.rdf_display())?;
        }

        Ok(())
    }
}

#[cfg(feature = "serde")]
impl<R: serde::Serialize> serde::Serialize for HashDataset<R> {
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
impl<'de, R: Clone + Eq + Hash + serde::Deserialize<'de>> serde::Deserialize<'de> for HashDataset<R> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<R>(std::marker::PhantomData<R>);

        impl<'de, R: Clone + Eq + Hash + serde::Deserialize<'de>> serde::de::Visitor<'de> for Visitor<R> {
            type Value = HashDataset<R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an RDF dataset")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut result = HashDataset::new();

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

    use super::HashDataset;

    fn rng_graph(rng: &mut SmallRng) -> Option<u32> {
        let g = rng.next_u32();
        if g % 2 == 0 { Some(g) } else { None }
    }

    fn insert_test(n: usize, seed: [u8; 32]) {
        let mut rng = SmallRng::from_seed(seed);
        let mut quads = Vec::new();
        quads.resize_with(n, || Quad(rng.next_u32(), rng.next_u32(), rng.next_u32(), rng_graph(&mut rng)));

        let mut dataset = HashDataset::new();
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

        let mut dataset = HashDataset::new();
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
