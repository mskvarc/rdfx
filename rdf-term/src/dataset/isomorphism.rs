use std::{
    collections::{BTreeMap, BTreeSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

use crate::{LocalTerm, Quad, dataset::TraversableDataset, interpretation::ReverseInterpretation};

const SELF_TAG: u64 = 0xA5A5_5A5A_DEAD_BEEF;
const ANON_TAG: u64 = 0x1234_5678_9ABC_DEF0;
const MAX_REFINEMENT_ROUNDS: usize = 8;

/// Checks that there is an isomorphism between the datasets `a` and `b`.
///
/// There is an isomorphism if there exists a blank node identifier bijection
/// between `a` and `b`.
/// This is equivalent to `find_bijection_with(a, b).is_some()`.
pub fn are_isomorphic<A, B>(a: &A, b: &B) -> bool
where
    A: TraversableDataset<Resource = LocalTerm>,
    B: TraversableDataset<Resource = LocalTerm>,
{
    are_isomorphic_with(&(), a, b)
}

/// Checks that there is an isomorphism between the datasets `a` and `b`.
///
/// There is an isomorphism if there exists a blank node identifier bijection
/// between `a` and `b`.
/// This is equivalent to `find_bijection_with(a, b).is_some()`.
pub fn are_isomorphic_with<I, A, B>(interpretation: &I, a: &A, b: &B) -> bool
where
    I: ReverseInterpretation,
    I::Resource: Ord,
    A: TraversableDataset<Resource = I::Resource>,
    B: TraversableDataset<Resource = I::Resource>,
{
    find_bijection_with(interpretation, a, b).is_some()
}

/// Finds a blank node identifier bijection between from `a` to `b`.
/// If such bijection exists,
/// there is an isomorphism between `a` and `b`.
pub fn find_bijection<'a, 'b, A, B>(a: &'a A, b: &'b B) -> Option<BTreeBijection<'a, 'b>>
where
    A: TraversableDataset<Resource = LocalTerm>,
    B: TraversableDataset<Resource = LocalTerm>,
{
    find_bijection_with(&(), a, b)
}

/// Finds a blank node identifier bijection between from `a` to `b`.
/// If such bijection exists,
/// there is an isomorphism between `a` and `b`.
pub fn find_bijection_with<'a, 'b, I, A, B>(interpretation: &I, a: &'a A, b: &'b B) -> Option<BTreeBijection<'a, 'b, I::Resource>>
where
    I: ReverseInterpretation,
    I::Resource: Ord,
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

    // Step 1: collect signatures.
    let mut a_blanks_map = BTreeMap::new();
    let mut b_blanks_map = BTreeMap::new();
    collect_signatures(interpretation, &mut a_blanks_map, a);
    collect_signatures(interpretation, &mut b_blanks_map, b);

    if a_blanks_map.len() != b_blanks_map.len() {
        return None;
    }

    // Step 2: refine blank-node colors via Weisfeiler-Leman style iterations.
    let (a_color, b_color) = refine_colors(interpretation, &a_blanks_map, &b_blanks_map)?;

    // Step 3: build per-color groups in B and find candidates for each A blank.
    let mut b_by_color: BTreeMap<u64, Vec<&'b I::Resource>> = BTreeMap::new();
    for (b_blank, &c) in &b_color {
        b_by_color.entry(c).or_default().push(*b_blank);
    }

    let mut candidates: Vec<(&'a I::Resource, BTreeSet<&'b I::Resource>)> = Vec::new();
    for (a_blank_id, &c) in &a_color {
        let same_color = b_by_color.get(&c)?;
        let a_sig = a_blanks_map.get(a_blank_id).unwrap();
        let mut a_blank_id_candidates = BTreeSet::new();
        for &b_blank_id in same_color {
            let b_sig = b_blanks_map.get(b_blank_id).unwrap();
            if a_sig.matches(interpretation, b_sig) {
                a_blank_id_candidates.insert(b_blank_id);
            }
        }

        if a_blank_id_candidates.is_empty() {
            return None;
        }

        candidates.push((*a_blank_id, a_blank_id_candidates));
    }

    let mut bijection = BTreeBijection::new();
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

fn collect_signatures<'d, I, D>(interpretation: &I, map: &mut BTreeMap<&'d I::Resource, BlankSignature<'d, I::Resource>>, ds: &'d D)
where
    I: ReverseInterpretation,
    I::Resource: Ord,
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

    for sig in map.values_mut() {
        sig.0.sort_unstable();
    }
}

fn hash_one<T: Hash + ?Sized>(v: &T) -> u64 {
    let mut h = DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

fn nonblank_fingerprint<I>(interpretation: &I, r: &I::Resource) -> u64
where
    I: ReverseInterpretation,
{
    let mut iris: Vec<u64> = interpretation.iris_of(r).map(|i| hash_one(&i)).collect();
    let mut lits: Vec<u64> = interpretation.literals_of(r).map(|l| hash_one(&l)).collect();
    if iris.is_empty() && lits.is_empty() {
        // Anonymous resource encountered without a known color.
        return ANON_TAG;
    }
    iris.sort_unstable();
    lits.sort_unstable();
    let mut h = DefaultHasher::new();
    0x4Eu8.hash(&mut h);
    iris.hash(&mut h);
    lits.hash(&mut h);
    h.finish()
}

fn position_fingerprint<I, R>(interpretation: &I, term: &R, my_id: &R, color: &BTreeMap<&R, u64>) -> u64
where
    I: ReverseInterpretation<Resource = R>,
    R: Ord,
{
    if term == my_id {
        SELF_TAG
    } else if interpretation.is_anonymous(term) {
        color.get(term).copied().unwrap_or(ANON_TAG)
    } else {
        nonblank_fingerprint(interpretation, term)
    }
}

fn quad_fingerprint<I, R>(interpretation: &I, q: Quad<&R>, my_id: &R, color: &BTreeMap<&R, u64>) -> u64
where
    I: ReverseInterpretation<Resource = R>,
    R: Ord,
{
    let mut h = DefaultHasher::new();
    position_fingerprint(interpretation, q.0, my_id, color).hash(&mut h);
    position_fingerprint(interpretation, q.1, my_id, color).hash(&mut h);
    position_fingerprint(interpretation, q.2, my_id, color).hash(&mut h);
    match q.3 {
        Some(g) => {
            1u8.hash(&mut h);
            position_fingerprint(interpretation, g, my_id, color).hash(&mut h);
        }
        None => 0u8.hash(&mut h),
    }
    h.finish()
}

fn compute_round<'d, I, R>(
    interpretation: &I,
    blanks: &BTreeMap<&'d R, BlankSignature<'d, R>>,
    color: &BTreeMap<&'d R, u64>,
) -> BTreeMap<&'d R, u64>
where
    I: ReverseInterpretation<Resource = R>,
    R: Ord,
{
    let mut out = BTreeMap::new();
    for (blank, sig) in blanks {
        let mut quad_fps: Vec<u64> = sig.0.iter().map(|q| quad_fingerprint(interpretation, *q, blank, color)).collect();
        quad_fps.sort_unstable();
        let mut h = DefaultHasher::new();
        color.get(blank).copied().unwrap_or(0).hash(&mut h);
        quad_fps.hash(&mut h);
        out.insert(*blank, h.finish());
    }
    out
}

fn color_multiset<R>(color: &BTreeMap<&R, u64>) -> BTreeMap<u64, usize> {
    let mut m: BTreeMap<u64, usize> = BTreeMap::new();
    for &c in color.values() {
        *m.entry(c).or_default() += 1;
    }
    m
}

#[allow(clippy::type_complexity)]
fn refine_colors<'da, 'db, I, R>(
    interpretation: &I,
    a_blanks: &BTreeMap<&'da R, BlankSignature<'da, R>>,
    b_blanks: &BTreeMap<&'db R, BlankSignature<'db, R>>,
) -> Option<(BTreeMap<&'da R, u64>, BTreeMap<&'db R, u64>)>
where
    I: ReverseInterpretation<Resource = R>,
    R: Ord,
{
    let mut a_color: BTreeMap<&R, u64> = a_blanks.keys().map(|k| (*k, 0u64)).collect();
    let mut b_color: BTreeMap<&R, u64> = b_blanks.keys().map(|k| (*k, 0u64)).collect();

    let mut prev_unique = 1usize;
    for _ in 0..MAX_REFINEMENT_ROUNDS {
        let new_a = compute_round(interpretation, a_blanks, &a_color);
        let new_b = compute_round(interpretation, b_blanks, &b_color);
        let a_counts = color_multiset(&new_a);
        let b_counts = color_multiset(&new_b);
        if a_counts != b_counts {
            return None;
        }
        let unique = a_counts.len();
        a_color = new_a;
        b_color = new_b;
        if unique == a_blanks.len() || unique == prev_unique {
            break;
        }
        prev_unique = unique;
    }
    Some((a_color, b_color))
}

/// Blank node identifier bijection
/// between two (isomorphic) datasets.
pub struct BTreeBijection<'a, 'b, R = LocalTerm> {
    pub forward: BTreeMap<&'a R, &'b R>,
    pub backward: BTreeMap<&'b R, &'a R>,
}

impl<R> BTreeBijection<'_, '_, R> {
    fn new() -> Self {
        Self {
            forward: BTreeMap::new(),
            backward: BTreeMap::new(),
        }
    }
}

impl<'a, 'b, R: Ord> BTreeBijection<'a, 'b, R> {
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
        candidates: &[(&'a R, BTreeSet<&'b R>)],
        depth: usize,
        a: &BTreeMap<&'a R, BlankSignature<'a, R>>,
        b: &BTreeMap<&'b R, BlankSignature<'b, R>>,
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

/// Signature of a blank node identifier.
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
