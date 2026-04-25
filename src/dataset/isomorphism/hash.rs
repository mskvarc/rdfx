use std::hash::Hash;

use hashbrown::hash_map::Entry;

use crate::{
    LocalTerm,
    Quad,
    dataset::TraversableDataset,
    interpretation::ReverseInterpretation,
    utils::hash_map::{HashMap, HashSet},
};

/// Checks that there is an isomorphism between the datasets `a` and `b`.
///
/// There is an isomorphism if there exists a blank node identifier bijection
/// between `a` and `b`.
/// This is equivalent to `find_hash_bijection_with(a, b).is_some()`.
pub fn are_isomorphic_hash<A, B>(a: &A, b: &B) -> bool
where
    A: TraversableDataset<Resource = LocalTerm>,
    B: TraversableDataset<Resource = LocalTerm>,
{
    are_isomorphic_hash_with(&(), a, b)
}

/// Checks that there is an isomorphism between the datasets `a` and `b`.
///
/// There is an isomorphism if there exists a blank node identifier bijection
/// between `a` and `b`.
pub fn are_isomorphic_hash_with<I, A, B>(interpretation: &I, a: &A, b: &B) -> bool
where
    I: ReverseInterpretation,
    I::Resource: Eq + Hash,
    A: TraversableDataset<Resource = I::Resource>,
    B: TraversableDataset<Resource = I::Resource>,
{
    find_hash_bijection_with(interpretation, a, b).is_some()
}

/// Finds a blank node identifier bijection from `a` to `b`.
/// If such bijection exists, there is an isomorphism between `a` and `b`.
pub fn find_hash_bijection<'a, 'b, A, B>(a: &'a A, b: &'b B) -> Option<HashBijection<'a, 'b>>
where
    A: TraversableDataset<Resource = LocalTerm>,
    B: TraversableDataset<Resource = LocalTerm>,
{
    find_hash_bijection_with(&(), a, b)
}

/// Finds a blank node identifier bijection from `a` to `b`.
/// If such bijection exists, there is an isomorphism between `a` and `b`.
pub fn find_hash_bijection_with<'a, 'b, I, A, B>(interpretation: &I, a: &'a A, b: &'b B) -> Option<HashBijection<'a, 'b, I::Resource>>
where
    I: ReverseInterpretation,
    I::Resource: Eq + Hash,
    A: TraversableDataset<Resource = I::Resource>,
    B: TraversableDataset<Resource = I::Resource>,
{
    if a.quads_count() != b.quads_count() {
        return None;
    }

    let a_blank_count = a.quads().fold(0, |c, q| c + blank_count(interpretation, q));
    let b_blank_count = b.quads().fold(0, |c, q| c + blank_count(interpretation, q));

    if a_blank_count != b_blank_count {
        return None;
    }

    let mut a_blanks_map: HashMap<&I::Resource, BlankSignature<I::Resource>> = HashMap::default();
    let mut b_blanks_map: HashMap<&I::Resource, BlankSignature<I::Resource>> = HashMap::default();
    collect_signatures(interpretation, &mut a_blanks_map, a);
    collect_signatures(interpretation, &mut b_blanks_map, b);

    if a_blanks_map.len() != b_blanks_map.len() {
        return None;
    }

    let a_groups = split_by_size(&a_blanks_map);
    let b_groups = split_by_size(&b_blanks_map);

    if a_groups.len() != b_groups.len() {
        return None;
    }

    if !a_groups.iter().all(|(len, _)| b_groups.contains_key(len)) {
        return None;
    }

    let mut candidates: Vec<(&'a I::Resource, HashSet<&'b I::Resource>)> = Vec::new();
    for (len, a_group) in a_groups {
        let b_group = b_groups.get(&len).unwrap();

        for (a_blank_id, a_sig) in a_group {
            let mut a_blank_id_candidates: HashSet<&'b I::Resource> = HashSet::default();
            for (b_blank_id, b_sig) in b_group {
                if a_sig.matches(interpretation, b_sig) {
                    a_blank_id_candidates.insert(*b_blank_id);
                }
            }

            if a_blank_id_candidates.is_empty() {
                return None;
            }

            candidates.push((a_blank_id, a_blank_id_candidates));
        }
    }

    candidates.sort_by_key(|(_, set)| set.len());

    let mut bijection = HashBijection::new();
    if bijection.find_from_candidates(interpretation, &candidates, 0, &a_blanks_map, &b_blanks_map) {
        Some(bijection)
    } else {
        None
    }
}

fn resource_matches<I>(interpretation: &I, a: &I::Resource, b: &I::Resource) -> bool
where
    I: ReverseInterpretation,
{
    for a in interpretation.iris_of(a) {
        for b in interpretation.iris_of(b) {
            if a == b {
                return true;
            }
        }
    }

    for a in interpretation.literals_of(a) {
        for b in interpretation.literals_of(b) {
            if a == b {
                return true;
            }
        }
    }

    interpretation.is_anonymous(a) && interpretation.is_anonymous(b)
}

fn quad_matches<I>(interpretation: &I, a: Quad<&I::Resource>, b: Quad<&I::Resource>) -> bool
where
    I: ReverseInterpretation,
{
    resource_matches(interpretation, a.0, b.0)
        && resource_matches(interpretation, a.1, b.1)
        && resource_matches(interpretation, a.2, b.2)
        && match (a.3, b.3) {
            (Some(a), Some(b)) => resource_matches(interpretation, a, b),
            (None, None) => true,
            _ => false,
        }
}

fn blank_count<I>(interpretation: &I, Quad(s, p, o, g): Quad<&I::Resource>) -> usize
where
    I: ReverseInterpretation,
{
    let mut r = 0;

    if interpretation.is_anonymous(s) {
        r += 1
    }

    if interpretation.is_anonymous(p) {
        r += 1
    }

    if interpretation.is_anonymous(o) {
        r += 1
    }

    if let Some(g) = g {
        if interpretation.is_anonymous(g) {
            r += 1
        }
    }

    r
}

fn collect_signatures<'d, I, D>(interpretation: &I, map: &mut HashMap<&'d I::Resource, BlankSignature<'d, I::Resource>>, ds: &'d D)
where
    I: ReverseInterpretation,
    I::Resource: Eq + Hash,
    D: TraversableDataset<Resource = I::Resource>,
{
    for quad in ds.quads() {
        if interpretation.is_anonymous(quad.0) {
            map.entry(quad.0).or_default().insert(quad);
        }

        if interpretation.is_anonymous(quad.1) {
            map.entry(quad.1).or_default().insert(quad);
        }

        if interpretation.is_anonymous(quad.2) {
            map.entry(quad.2).or_default().insert(quad);
        }

        if let Some(g) = quad.3 {
            if interpretation.is_anonymous(g) {
                map.entry(g).or_default().insert(quad);
            }
        }
    }
}

fn split_by_size<'s, 'd, R>(blanks: &'s HashMap<&'d R, BlankSignature<'d, R>>) -> HashMap<usize, HashMap<&'d R, &'s BlankSignature<'d, R>>>
where
    R: Eq + Hash,
{
    let mut result: HashMap<usize, HashMap<&'d R, &'s BlankSignature<'d, R>>> = HashMap::default();

    for (&blank_id, sig) in blanks {
        match result.entry(sig.len()) {
            Entry::Vacant(entry) => {
                let mut map: HashMap<&'d R, &'s BlankSignature<'d, R>> = HashMap::default();
                map.insert(blank_id, sig);
                entry.insert(map);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(blank_id, sig);
            }
        }
    }

    result
}

/// Blank node identifier bijection between two (isomorphic) datasets.
pub struct HashBijection<'a, 'b, R = LocalTerm> {
    pub forward: HashMap<&'a R, &'b R>,
    pub backward: HashMap<&'b R, &'a R>,
}

impl<R> HashBijection<'_, '_, R> {
    fn new() -> Self {
        Self {
            forward: HashMap::default(),
            backward: HashMap::default(),
        }
    }
}

impl<'a, 'b, R: Eq + Hash> HashBijection<'a, 'b, R> {
    fn resource_matches_with<I>(&self, interpretation: &I, a: &'a R, b: &'b R) -> bool
    where
        I: ReverseInterpretation<Resource = R>,
    {
        for a in interpretation.iris_of(a) {
            for b in interpretation.iris_of(b) {
                if a == b {
                    return true;
                }
            }
        }

        for a in interpretation.literals_of(a) {
            for b in interpretation.literals_of(b) {
                if a == b {
                    return true;
                }
            }
        }

        match self.forward.get(a) {
            Some(&c) => c == b,
            None => match self.backward.get(b) {
                Some(&c) => a == c,
                None => true,
            },
        }
    }

    fn quad_matches_with<I>(&self, interpretation: &I, a: Quad<&'a R>, b: Quad<&'b R>) -> bool
    where
        I: ReverseInterpretation<Resource = R>,
    {
        self.resource_matches_with(interpretation, a.0, b.0)
            && self.resource_matches_with(interpretation, a.1, b.1)
            && self.resource_matches_with(interpretation, a.2, b.2)
            && match (a.3, b.3) {
                (Some(a), Some(b)) => self.resource_matches_with(interpretation, a, b),
                (None, None) => true,
                _ => false,
            }
    }

    fn signature_matches_with<I>(&self, interpretation: &I, a: &BlankSignature<'a, R>, b: &BlankSignature<'b, R>) -> bool
    where
        I: ReverseInterpretation<Resource = R>,
    {
        match_quad_lists(&a.0, &b.0, |qa, qb| self.quad_matches_with(interpretation, qa, qb))
    }

    fn find_from_candidates<I>(
        &mut self,
        interpretation: &I,
        candidates: &[(&'a R, HashSet<&'b R>)],
        depth: usize,
        a: &HashMap<&'a R, BlankSignature<'a, R>>,
        b: &HashMap<&'b R, BlankSignature<'b, R>>,
    ) -> bool
    where
        I: ReverseInterpretation<Resource = R>,
    {
        let Some((a_blank_id, b_candidates)) = candidates.get(depth) else {
            return true;
        };

        for b_candidate in b_candidates {
            if self.backward.contains_key(b_candidate) {
                continue;
            }

            self.forward.insert(a_blank_id, b_candidate);
            self.backward.insert(b_candidate, a_blank_id);

            let sig_ok = self.signature_matches_with(interpretation, a.get(a_blank_id).unwrap(), b.get(b_candidate).unwrap());
            if sig_ok && self.find_from_candidates(interpretation, candidates, depth + 1, a, b) {
                return true;
            }

            self.forward.remove(a_blank_id);
            self.backward.remove(b_candidate);
        }

        false
    }
}

#[allow(clippy::type_complexity)]
struct BlankSignature<'a, R>(Vec<Quad<&'a R>>);

impl<R> Default for BlankSignature<'_, R> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<'a, R> BlankSignature<'a, R> {
    fn insert(&mut self, quad: Quad<&'a R>) {
        self.0.push(quad)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn matches<I>(&self, interpretation: &I, other: &BlankSignature<R>) -> bool
    where
        I: ReverseInterpretation<Resource = R>,
    {
        match_quad_lists(&self.0, &other.0, |qa, qb| quad_matches(interpretation, qa, qb))
    }
}

fn match_quad_lists<R, F>(a: &[Quad<&R>], b: &[Quad<&R>], mut matches: F) -> bool
where
    F: FnMut(Quad<&R>, Quad<&R>) -> bool,
{
    if a.len() != b.len() {
        return false;
    }
    let n = b.len();
    if n <= 128 {
        let mut available: u128 = if n == 128 { u128::MAX } else { (1u128 << n) - 1 };
        for &qa in a {
            let mut remaining = available;
            let mut hit = false;
            while remaining != 0 {
                let i = remaining.trailing_zeros() as usize;
                let bit = 1u128 << i;
                remaining &= !bit;
                if matches(qa, b[i]) {
                    available &= !bit;
                    hit = true;
                    break;
                }
            }
            if !hit {
                return false;
            }
        }
        true
    } else {
        let words = n.div_ceil(64);
        let mut available = vec![0u64; words];
        for (w, slot) in available.iter_mut().enumerate() {
            let bits = (n - w * 64).min(64);
            *slot = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
        }
        for &qa in a {
            let mut hit = false;
            'outer: for (w, slot) in available.iter_mut().enumerate() {
                let mut remaining = *slot;
                while remaining != 0 {
                    let bit = remaining.trailing_zeros();
                    let mask = 1u64 << bit;
                    remaining &= !mask;
                    let i = w * 64 + bit as usize;
                    if matches(qa, b[i]) {
                        *slot &= !mask;
                        hit = true;
                        break 'outer;
                    }
                }
            }
            if !hit {
                return false;
            }
        }
        true
    }
}
