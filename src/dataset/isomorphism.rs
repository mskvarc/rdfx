//! Blank-node bijection between datasets — RDF 1.1 §3.6 / §4.4 graph and
//! dataset equivalence under blank-node renaming.
//!
//! Per [RDF 1.1 Concepts §3.6][s3] / [§4.4][s4], two graphs (datasets) are
//! equivalent iff there exists a bijection between their blank node
//! identifiers that maps one to the other. [`find_bijection`] computes that
//! bijection if it exists; [`graph_equivalent`] / [`dataset_equivalent`] are
//! convenience predicates.
//!
//! Implementation: signature-bucket pre-filter (split by neighbourhood
//! signature size, then by per-quad match-set) followed by augmenting-path
//! bipartite matching. Not RDFC-1.0; just structural bijection.
//!
//! [s3]: https://www.w3.org/TR/rdf11-concepts/#section-graph-equality
//! [s4]: https://www.w3.org/TR/rdf11-concepts/#section-dataset-equality
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use crate::{
    Quad,
    Resource,
    dataset::{TraversableDataset, TraversableGraph},
    interpretation::ReverseInterpretation,
};

/// RDF 1.1 §3.6 graph equivalence: graphs are equal iff a blank-node
/// bijection exists between them. Wraps [`find_bijection_graph_with`].
pub fn graph_equivalent<I, A, B>(interpretation: &I, a: &A, b: &B) -> bool
where
    I: ReverseInterpretation,
    I::Resource: Resource + Ord,
    A: TraversableGraph<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource>,
    B: TraversableGraph<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource>,
{
    are_isomorphic_graph_with(interpretation, a, b)
}

/// RDF 1.1 §4.4 dataset equivalence: datasets are equal iff a blank-node
/// bijection exists between them. Wraps [`find_bijection_with`].
pub fn dataset_equivalent<I, A, B>(interpretation: &I, a: &A, b: &B) -> bool
where
    I: ReverseInterpretation,
    I::Resource: Resource + Ord,
    A: TraversableDataset<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource, Graph = I::Resource>,
    B: TraversableDataset<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource, Graph = I::Resource>,
{
    are_isomorphic_with(interpretation, a, b)
}

/// Checks that there is an isomorphism between the datasets `a` and `b`.
pub fn are_isomorphic_with<I, A, B>(interpretation: &I, a: &A, b: &B) -> bool
where
    I: ReverseInterpretation,
    I::Resource: Resource + Ord,
    A: TraversableDataset<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource, Graph = I::Resource>,
    B: TraversableDataset<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource, Graph = I::Resource>,
{
    find_bijection_with(interpretation, a, b).is_some()
}

/// Graph-flavoured variant of [`are_isomorphic_with`].
pub fn are_isomorphic_graph_with<I, A, B>(interpretation: &I, a: &A, b: &B) -> bool
where
    I: ReverseInterpretation,
    I::Resource: Resource + Ord,
    A: TraversableGraph<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource>,
    B: TraversableGraph<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource>,
{
    find_bijection_graph_with(interpretation, a, b).is_some()
}

/// Finds a blank node identifier bijection from `a` to `b`.
/// If such bijection exists, there is an isomorphism between `a` and `b`.
pub fn find_bijection_with<'a, 'b, I, A, B>(interpretation: &I, a: &'a A, b: &'b B) -> Option<Bijection<'a, 'b, I::Resource>>
where
    I: ReverseInterpretation,
    I::Resource: Resource + Ord,
    A: TraversableDataset<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource, Graph = I::Resource>,
    B: TraversableDataset<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource, Graph = I::Resource>,
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

    // Step 2: split by sizes.
    let a_groups = split_by_size(&a_blanks_map);
    let b_groups = split_by_size(&b_blanks_map);

    if a_groups.len() != b_groups.len() {
        return None;
    }

    if !a_groups.iter().all(|(len, _)| b_groups.contains_key(len)) {
        return None;
    }

    // Step 3: find candidates for each blank id.
    let mut candidates: Vec<(&'a I::Resource, BTreeSet<&'b I::Resource>)> = Vec::new();
    for (len, a_group) in a_groups {
        let b_group = &b_groups[&len];

        for (a_blank_id, a_sig) in a_group {
            let mut a_blank_id_candidates = BTreeSet::new();
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

    // Order by candidate-set size ascending: smallest sets first prunes the search tree fastest.
    candidates.sort_by_key(|(_, set)| set.len());

    let mut bijection = Bijection::new();
    if bijection.find_from_candidates(interpretation, &candidates, 0, &a_blanks_map, &b_blanks_map) {
        Some(bijection)
    } else {
        None
    }
}

/// Graph-flavoured variant of [`find_bijection_with`]. Wraps each triple as a
/// quad with default graph (`None`) and reuses the dataset bijection logic.
pub fn find_bijection_graph_with<'a, 'b, I, A, B>(interpretation: &I, a: &'a A, b: &'b B) -> Option<Bijection<'a, 'b, I::Resource>>
where
    I: ReverseInterpretation,
    I::Resource: Resource + Ord,
    A: TraversableGraph<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource>,
    B: TraversableGraph<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource>,
{
    // Project triples as quads with default graph (None).
    let a_quads: Vec<Quad<&I::Resource, &I::Resource, &I::Resource, &I::Resource>> = a.triples().map(|t| Quad(t.0, t.1, t.2, None)).collect();
    let b_quads: Vec<Quad<&I::Resource, &I::Resource, &I::Resource, &I::Resource>> = b.triples().map(|t| Quad(t.0, t.1, t.2, None)).collect();

    if a_quads.len() != b_quads.len() {
        return None;
    }

    let a_blank_count = a_quads.iter().fold(0, |c, q| c + blank_count(interpretation, *q));
    let b_blank_count = b_quads.iter().fold(0, |c, q| c + blank_count(interpretation, *q));

    if a_blank_count != b_blank_count {
        return None;
    }

    let mut a_blanks_map: BTreeMap<&I::Resource, BlankSignature<I::Resource>> = BTreeMap::new();
    let mut b_blanks_map: BTreeMap<&I::Resource, BlankSignature<I::Resource>> = BTreeMap::new();
    for q in &a_quads {
        let q = *q;
        if interpretation.is_blank_id(q.0) {
            a_blanks_map.entry(q.0).or_default().insert(q);
        }
        if interpretation.is_blank_id(q.1) {
            a_blanks_map.entry(q.1).or_default().insert(q);
        }
        if interpretation.is_blank_id(q.2) {
            a_blanks_map.entry(q.2).or_default().insert(q);
        }
    }
    for q in &b_quads {
        let q = *q;
        if interpretation.is_blank_id(q.0) {
            b_blanks_map.entry(q.0).or_default().insert(q);
        }
        if interpretation.is_blank_id(q.1) {
            b_blanks_map.entry(q.1).or_default().insert(q);
        }
        if interpretation.is_blank_id(q.2) {
            b_blanks_map.entry(q.2).or_default().insert(q);
        }
    }

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

    let mut candidates: Vec<(&'a I::Resource, BTreeSet<&'b I::Resource>)> = Vec::new();
    for (len, a_group) in a_groups {
        let b_group = &b_groups[&len];

        for (a_blank_id, a_sig) in a_group {
            let mut a_blank_id_candidates = BTreeSet::new();
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

    let mut bijection = Bijection::new();
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

    interpretation.is_blank_id(a) && interpretation.is_blank_id(b)
}

fn quad_matches<I>(interpretation: &I, a: Quad<&I::Resource, &I::Resource, &I::Resource, &I::Resource>, b: Quad<&I::Resource, &I::Resource, &I::Resource, &I::Resource>) -> bool
where
    I: ReverseInterpretation,
    I::Resource: Resource,
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

fn blank_count<I>(interpretation: &I, Quad(s, p, o, g): Quad<&I::Resource, &I::Resource, &I::Resource, &I::Resource>) -> usize
where
    I: ReverseInterpretation,
    I::Resource: Resource,
{
    let mut r = 0;

    if interpretation.is_blank_id(s) {
        r += 1
    }

    if interpretation.is_blank_id(p) {
        r += 1
    }

    if interpretation.is_blank_id(o) {
        r += 1
    }

    if let Some(g) = g {
        if interpretation.is_blank_id(g) {
            r += 1
        }
    }

    r
}

fn collect_signatures<'d, I, D>(interpretation: &I, map: &mut BTreeMap<&'d I::Resource, BlankSignature<'d, I::Resource>>, ds: &'d D)
where
    I: ReverseInterpretation,
    I::Resource: Resource + Ord,
    D: TraversableDataset<Subject = I::Resource, Predicate = I::Resource, Object = I::Resource, Graph = I::Resource>,
{
    for quad in ds.quads() {
        if interpretation.is_blank_id(quad.0) {
            map.entry(quad.0).or_default().insert(quad);
        }

        if interpretation.is_blank_id(quad.1) {
            map.entry(quad.1).or_default().insert(quad);
        }

        if interpretation.is_blank_id(quad.2) {
            map.entry(quad.2).or_default().insert(quad);
        }

        if let Some(g) = quad.3 {
            if interpretation.is_blank_id(g) {
                map.entry(g).or_default().insert(quad);
            }
        }
    }
}

fn split_by_size<'s, 'd, R>(blanks: &'s BTreeMap<&'d R, BlankSignature<'d, R>>) -> BTreeMap<usize, BTreeMap<&'d R, &'s BlankSignature<'d, R>>>
where
    R: Resource + Ord,
{
    let mut result = BTreeMap::new();

    for (&blank_id, sig) in blanks {
        match result.entry(sig.len()) {
            Entry::Vacant(entry) => {
                let mut map = BTreeMap::new();
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

/// Blank node identifier bijection
/// between two (isomorphic) datasets.
pub struct Bijection<'a, 'b, R> {
    pub forward: BTreeMap<&'a R, &'b R>,
    pub backward: BTreeMap<&'b R, &'a R>,
}

impl<R> Bijection<'_, '_, R> {
    const fn new() -> Self {
        Self {
            forward: BTreeMap::new(),
            backward: BTreeMap::new(),
        }
    }
}

impl<'a, 'b, R: Resource + Ord> Bijection<'a, 'b, R> {
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

    fn quad_matches_with<I>(&self, interpretation: &I, a: Quad<&'a R, &'a R, &'a R, &'a R>, b: Quad<&'b R, &'b R, &'b R, &'b R>) -> bool
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

            let sig_ok = if depth == 0 {
                true
            } else {
                match (a.get(a_blank_id), b.get(b_candidate)) {
                    (Some(a_sig), Some(b_sig)) => self.signature_matches_with(interpretation, a_sig, b_sig),
                    _ => false,
                }
            };
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
struct BlankSignature<'a, R: Resource>(Vec<Quad<&'a R, &'a R, &'a R, &'a R>>);

impl<R: Resource> Default for BlankSignature<'_, R> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<'a, R: Resource> BlankSignature<'a, R> {
    fn insert(&mut self, quad: Quad<&'a R, &'a R, &'a R, &'a R>) {
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

fn match_quad_lists<R: Resource, F>(a: &[Quad<&R, &R, &R, &R>], b: &[Quad<&R, &R, &R, &R>], mut matches: F) -> bool
where
    F: FnMut(Quad<&R, &R, &R, &R>, Quad<&R, &R, &R, &R>) -> bool,
{
    if a.len() != b.len() {
        return false;
    }
    let n = b.len();
    let mut matched_b: Vec<Option<usize>> = vec![None; n];
    for ai in 0..a.len() {
        let mut visited = vec![false; n];
        if !try_augment(ai, a, b, &mut matched_b, &mut visited, &mut matches) {
            return false;
        }
    }
    true
}

fn try_augment<R: Resource, F>(
    ai: usize,
    a: &[Quad<&R, &R, &R, &R>],
    b: &[Quad<&R, &R, &R, &R>],
    matched_b: &mut [Option<usize>],
    visited: &mut [bool],
    matches: &mut F,
) -> bool
where
    F: FnMut(Quad<&R, &R, &R, &R>, Quad<&R, &R, &R, &R>) -> bool,
{
    for bi in 0..b.len() {
        if visited[bi] {
            continue;
        }
        if !matches(a[ai], b[bi]) {
            continue;
        }
        visited[bi] = true;
        let augmented = match matched_b[bi] {
            None => true,
            Some(prev_ai) => try_augment(prev_ai, a, b, matched_b, visited, matches),
        };
        if augmented {
            matched_b[bi] = Some(ai);
            return true;
        }
    }
    false
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn bipartite_matching_adversarial() {
        // a = [x, y], b = [X, Y]. x can match {X, Y}; y can match {X} only.
        // Greedy first-fit picks x->X, then y has no candidate. Augmenting-path
        // recovers the true matching x->Y, y->X.
        let x = "x";
        let y = "y";
        let cap_x = "X";
        let cap_y = "Y";
        let r = "r";
        let z = "z";

        let a: Vec<Quad<&&str, &&str, &&str, &&str>> = vec![Quad(&x, &r, &z, None), Quad(&y, &r, &z, None)];
        let b: Vec<Quad<&&str, &&str, &&str, &&str>> = vec![Quad(&cap_x, &r, &z, None), Quad(&cap_y, &r, &z, None)];

        let result = match_quad_lists(&a, &b, |qa, qb| {
            // x matches both X and Y; y matches only X.
            if *qa.0 == "x" {
                return *qb.0 == "X" || *qb.0 == "Y";
            }
            if *qa.0 == "y" {
                return *qb.0 == "X";
            }
            false
        });

        assert!(result, "augmenting-path matcher must find x->Y, y->X");
    }

    #[test]
    fn bipartite_matching_no_solution() {
        // a = [x, y], b = [X, Y]. Both x and y can only match X (not Y).
        // No perfect matching exists.
        let x = "x";
        let y = "y";
        let cap_x = "X";
        let cap_y = "Y";
        let r = "r";
        let z = "z";

        let a: Vec<Quad<&&str, &&str, &&str, &&str>> = vec![Quad(&x, &r, &z, None), Quad(&y, &r, &z, None)];
        let b: Vec<Quad<&&str, &&str, &&str, &&str>> = vec![Quad(&cap_x, &r, &z, None), Quad(&cap_y, &r, &z, None)];

        let result = match_quad_lists(&a, &b, |qa, qb| (*qa.0 == "x" || *qa.0 == "y") && *qb.0 == "X");
        assert!(!result, "no perfect matching when both a-nodes only match same b-node");
    }
}
