//! RDF 1.2 unstar/restar round-trip tests.
//!
//! Uses a tiny `Vec`-backed `TestGraph` to satisfy the
//! `TraversableGraph<Subject=Id, Predicate=IriBuf, Object=LocalTerm>` /
//! `GraphMut<...>` shape required by `unstar_graph` / `restar_graph` —
//! the heterogeneous lexical alphabet has no built-in storage type.
#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use std::collections::HashSet;

use iri_rs::IriBuf;
use rdf_rs::{
    BlankIdBuf,
    Id,
    LexicalTriple,
    LocalTerm,
    RDF_OBJECT,
    RDF_PREDICATE,
    RDF_REIFIES,
    RDF_SUBJECT,
    RDF_TRIPLE_TERM,
    RDF_TYPE,
    Term,
    Triple,
    TripleTerm,
    dataset::{Graph, GraphMut, TraversableGraph},
    star::{UnstarOptions, restar_graph, unstar_graph},
    generator::Blank,
};

// ----- minimal test graph -----

#[derive(Default, Debug)]
struct TestGraph(Vec<LexicalTriple>);

impl Graph for TestGraph {
    type Subject = Id;
    type Predicate = IriBuf;
    type Object = LocalTerm;
}

struct TripleIter<'a>(std::slice::Iter<'a, LexicalTriple>);

impl<'a> Iterator for TripleIter<'a> {
    type Item = Triple<&'a Id, &'a IriBuf, &'a LocalTerm>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|t| Triple(&t.0, &t.1, &t.2))
    }
}

impl TraversableGraph for TestGraph {
    type Triples<'a> = TripleIter<'a> where Self: 'a;
    fn triples(&self) -> Self::Triples<'_> {
        TripleIter(self.0.iter())
    }
}

impl GraphMut for TestGraph {
    fn insert(&mut self, t: LexicalTriple) {
        self.0.push(t);
    }
    fn remove(&mut self, _t: Triple<&Id, &IriBuf, &LocalTerm>) {
        // not exercised by unstar/restar
    }
}

// ----- helpers -----

fn ex(s: &str) -> IriBuf {
    IriBuf::new(format!("http://example.org/{s}")).unwrap()
}

fn s_iri(s: &str) -> Id {
    Id::iri(ex(s))
}

fn o_iri(s: &str) -> LocalTerm {
    LocalTerm::iri(ex(s))
}

fn count_with_predicate(g: &TestGraph, p: &str) -> usize {
    g.0.iter().filter(|t| t.1.as_str() == p).count()
}

fn contains_triple_term_object(g: &TestGraph, expected_body: &TripleTerm) -> bool {
    g.0.iter().any(|t| matches!(&t.2, LocalTerm::Triple(b) if &**b == expected_body))
}

fn distinct_objects(g: &TestGraph) -> HashSet<String> {
    g.0.iter().map(|t| format!("{:?}", t.2)).collect()
}

// ----- tests -----

#[test]
fn unstar_emits_four_reification_triples_per_triple_term() {
    let inner: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let outer = Triple(s_iri("a"), ex("says"), LocalTerm::triple(inner));

    let src = TestGraph(vec![outer]);
    let mut dest = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut dest, &mut g, UnstarOptions::default());

    // 1 original (rewritten) + 4 reification triples
    assert_eq!(dest.0.len(), 5);
    assert_eq!(count_with_predicate(&dest, RDF_TYPE.as_str()), 1);
    assert_eq!(count_with_predicate(&dest, RDF_SUBJECT.as_str()), 1);
    assert_eq!(count_with_predicate(&dest, RDF_PREDICATE.as_str()), 1);
    assert_eq!(count_with_predicate(&dest, RDF_OBJECT.as_str()), 1);

    // The original triple's object is no longer a triple term.
    let original = dest.0.iter().find(|t| t.1.as_str() == "http://example.org/says").unwrap();
    assert!(!original.2.is_triple());
    assert!(matches!(original.2, LocalTerm::BlankId(_)));
}

#[test]
fn unstar_then_restar_round_trip() {
    let inner: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let outer = Triple(s_iri("a"), ex("says"), LocalTerm::triple(inner.clone()));

    let src = TestGraph(vec![outer.clone()]);
    let mut reified = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut reified, &mut g, UnstarOptions::default());

    let mut refolded = TestGraph::default();
    restar_graph(&reified, &mut refolded);

    assert_eq!(refolded.0.len(), 1, "reified form refolds back to a single triple");
    assert!(contains_triple_term_object(&refolded, &inner));
    assert_eq!(refolded.0[0].0, outer.0);
    assert_eq!(refolded.0[0].1.as_str(), outer.1.as_str());
}

#[test]
fn unstar_nested_triple_terms() {
    let innermost: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let middle: TripleTerm = Triple(s_iri("b"), ex("says"), LocalTerm::triple(innermost.clone()));
    let outer = Triple(s_iri("a"), ex("says"), LocalTerm::triple(middle.clone()));

    let src = TestGraph(vec![outer]);
    let mut reified = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut reified, &mut g, UnstarOptions::default());

    // 1 outer + 4 (middle reification) + 4 (innermost reification) = 9
    assert_eq!(reified.0.len(), 9);
    assert_eq!(count_with_predicate(&reified, RDF_TYPE.as_str()), 2);
    assert_eq!(count_with_predicate(&reified, RDF_SUBJECT.as_str()), 2);

    // Round-trip
    let mut refolded = TestGraph::default();
    restar_graph(&reified, &mut refolded);
    assert_eq!(refolded.0.len(), 1);
    let body = refolded.0[0].2.as_triple().unwrap();
    assert_eq!(body, &middle);
    let inner_body = body.2.as_triple().unwrap();
    assert_eq!(inner_body, &innermost);
}

#[test]
fn unstar_reuses_reifier_for_identical_triple_terms_when_enabled() {
    let body: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let t1 = Triple(s_iri("a"), ex("says"), LocalTerm::triple(body.clone()));
    let t2 = Triple(s_iri("b"), ex("says"), LocalTerm::triple(body));

    let src = TestGraph(vec![t1, t2]);
    let mut dest = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut dest, &mut g, UnstarOptions { reuse_reifier: true });

    // 2 originals + 4 reification (shared) = 6
    assert_eq!(dest.0.len(), 6);
    assert_eq!(count_with_predicate(&dest, RDF_TYPE.as_str()), 1);
}

#[test]
fn unstar_unique_reifier_per_occurrence_when_disabled() {
    let body: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let t1 = Triple(s_iri("a"), ex("says"), LocalTerm::triple(body.clone()));
    let t2 = Triple(s_iri("b"), ex("says"), LocalTerm::triple(body));

    let src = TestGraph(vec![t1, t2]);
    let mut dest = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut dest, &mut g, UnstarOptions { reuse_reifier: false });

    // 2 originals + 8 reification = 10
    assert_eq!(dest.0.len(), 10);
    assert_eq!(count_with_predicate(&dest, RDF_TYPE.as_str()), 2);
}

#[test]
fn restar_leaves_non_reifier_blanks_untouched() {
    let blank = LocalTerm::BlankId(BlankIdBuf::new("_:b0".to_owned()).unwrap());
    let t = Triple(s_iri("a"), ex("p"), blank.clone());
    let src = TestGraph(vec![t]);

    let mut dest = TestGraph::default();
    restar_graph(&src, &mut dest);

    assert_eq!(dest.0.len(), 1);
    assert_eq!(dest.0[0].2, blank);
}

#[test]
fn restar_leaves_incomplete_reifier_alone() {
    // Only `rdf:type rdf:TripleTerm` and `rdf:subject` — incomplete.
    let r = Id::BlankId(BlankIdBuf::new("_:r".to_owned()).unwrap());
    let triple_term_iri = LocalTerm::iri(IriBuf::from(RDF_TRIPLE_TERM));
    let src = TestGraph(vec![
        Triple(r.clone(), IriBuf::from(RDF_TYPE), triple_term_iri),
        Triple(r, IriBuf::from(RDF_SUBJECT), o_iri("s")),
        // Missing rdf:predicate, rdf:object
        Triple(s_iri("a"), ex("p"), LocalTerm::BlankId(BlankIdBuf::new("_:r".to_owned()).unwrap())),
    ]);

    let mut dest = TestGraph::default();
    restar_graph(&src, &mut dest);

    // All input copied through untouched (no refolding).
    assert_eq!(dest.0.len(), src.0.len());
}

#[test]
fn restar_preserves_unrelated_triples() {
    let inner: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let outer = Triple(s_iri("a"), ex("says"), LocalTerm::triple(inner.clone()));
    let unrelated = Triple(s_iri("z"), ex("p"), o_iri("o"));

    let src = TestGraph(vec![outer, unrelated.clone()]);
    let mut reified = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut reified, &mut g, UnstarOptions::default());

    let mut refolded = TestGraph::default();
    restar_graph(&reified, &mut refolded);

    assert_eq!(refolded.0.len(), 2);
    assert!(contains_triple_term_object(&refolded, &inner));
    assert!(refolded.0.iter().any(|t| t.0 == unrelated.0 && t.1.as_str() == unrelated.1.as_str()));
}

#[test]
fn rdf_reifies_constant_is_correct_iri() {
    assert_eq!(RDF_REIFIES.as_str(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies");
    assert_eq!(RDF_TRIPLE_TERM.as_str(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#TripleTerm");
}

#[test]
fn unstar_wraps_blank_subject_inside_triple_term() {
    let blank_subj = Id::BlankId(BlankIdBuf::new("_:inner".to_owned()).unwrap());
    let inner: TripleTerm = Triple(blank_subj, ex("p"), o_iri("o"));
    let outer = Triple(s_iri("a"), ex("says"), LocalTerm::triple(inner));

    let src = TestGraph(vec![outer]);
    let mut dest = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut dest, &mut g, UnstarOptions::default());

    // Find the rdf:subject triple, verify object is the blank from inner subject.
    let sub_triple = dest.0.iter().find(|t| t.1.as_str() == RDF_SUBJECT.as_str()).unwrap();
    assert!(matches!(sub_triple.2, LocalTerm::BlankId(ref b) if b.as_str() == "_:inner"));
}

#[test]
fn objects_after_round_trip_set_distinct_from_reified_intermediate() {
    // Documents that reified intermediate has different object set than refolded form.
    let inner: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let outer = Triple(s_iri("a"), ex("says"), LocalTerm::triple(inner));
    let src = TestGraph(vec![outer]);

    let mut reified = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut reified, &mut g, UnstarOptions::default());
    let mut refolded = TestGraph::default();
    restar_graph(&reified, &mut refolded);

    let reified_objects = distinct_objects(&reified);
    let refolded_objects = distinct_objects(&refolded);
    assert_ne!(reified_objects, refolded_objects);
}

#[test]
fn unused_term_namespace_stays_silent() {
    // Sanity: `Term` re-exported and importable.
    let _ = Term::Iri(ex("x"));
}

// ----- RDF 1.2 §6 rdf:reifies-aware unstar -----

#[test]
fn unstar_preserves_explicit_blank_reifier_from_rdf_reifies_annotation() {
    // Input:
    //   _:r rdf:reifies <<( s p o )>> .
    //   _:r ex:said-by <ex:alice> .
    let body: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let r_blank = BlankIdBuf::new("_:r".to_owned()).unwrap();
    let r_id = Id::BlankId(r_blank);

    let src = TestGraph(vec![
        Triple(r_id.clone(), IriBuf::from(RDF_REIFIES), LocalTerm::triple(body)),
        Triple(r_id, ex("said-by"), o_iri("alice")),
    ]);

    let mut dest = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut dest, &mut g, UnstarOptions::default());

    // Expect: 4 reification triples (subject = _:r) + 1 annotation = 5; no
    // rdf:reifies triple in output.
    assert_eq!(dest.0.len(), 5);
    assert_eq!(count_with_predicate(&dest, RDF_REIFIES.as_str()), 0);
    let reification_subjects: Vec<&Id> = dest
        .0
        .iter()
        .filter(|t| {
            let p = t.1.as_str();
            p == RDF_TYPE.as_str() || p == RDF_SUBJECT.as_str() || p == RDF_PREDICATE.as_str() || p == RDF_OBJECT.as_str()
        })
        .map(|t| &t.0)
        .collect();
    assert_eq!(reification_subjects.len(), 4);
    for s in &reification_subjects {
        assert!(matches!(s, Id::BlankId(b) if b.as_str() == "_:r"));
    }
    // The annotation triple is preserved with the same _:r subject.
    let annotation = dest.0.iter().find(|t| t.1.as_str() == "http://example.org/said-by").unwrap();
    assert!(matches!(annotation.0, Id::BlankId(ref b) if b.as_str() == "_:r"));
}

#[test]
fn unstar_preserves_iri_reifier_from_rdf_reifies_annotation() {
    // Per RDF 1.2 §6 reifiers may be IRIs.
    let body: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let r_iri = ex("named-reifier");

    let src = TestGraph(vec![
        Triple(Id::Iri(r_iri.clone()), IriBuf::from(RDF_REIFIES), LocalTerm::triple(body)),
        Triple(Id::Iri(r_iri.clone()), ex("said-by"), o_iri("alice")),
    ]);

    let mut dest = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut dest, &mut g, UnstarOptions::default());

    // All four reification triples must use the same IRI as subject.
    let four = dest
        .0
        .iter()
        .filter(|t| {
            let p = t.1.as_str();
            p == RDF_TYPE.as_str() || p == RDF_SUBJECT.as_str() || p == RDF_PREDICATE.as_str() || p == RDF_OBJECT.as_str()
        })
        .count();
    assert_eq!(four, 4);
    for t in dest.0.iter().filter(|t| {
        let p = t.1.as_str();
        p == RDF_TYPE.as_str() || p == RDF_SUBJECT.as_str() || p == RDF_PREDICATE.as_str() || p == RDF_OBJECT.as_str()
    }) {
        assert!(matches!(&t.0, Id::Iri(i) if i.as_str() == r_iri.as_str()));
    }
}

#[test]
fn unstar_shares_reifier_between_rdf_reifies_and_object_occurrence() {
    // Body appears in two roles:
    //   _:r rdf:reifies <<( s p o )>>          (annotation)
    //   <ex:a> <ex:says> <<( s p o )>>          (regular triple-term object)
    // Both should map to the same reifier (_:r).
    let body: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let r_blank = BlankIdBuf::new("_:r".to_owned()).unwrap();
    let r_id = Id::BlankId(r_blank);

    let src = TestGraph(vec![
        Triple(r_id, IriBuf::from(RDF_REIFIES), LocalTerm::triple(body.clone())),
        Triple(s_iri("a"), ex("says"), LocalTerm::triple(body)),
    ]);

    let mut dest = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut dest, &mut g, UnstarOptions::default());

    // 4 reification (shared via _:r) + 1 rewritten regular triple = 5.
    assert_eq!(dest.0.len(), 5);
    assert_eq!(count_with_predicate(&dest, RDF_TYPE.as_str()), 1);
    // The regular triple's object is _:r.
    let says = dest.0.iter().find(|t| t.1.as_str() == "http://example.org/says").unwrap();
    assert!(matches!(&says.2, LocalTerm::BlankId(b) if b.as_str() == "_:r"));
}

#[test]
fn unstar_shares_explicit_reifier_even_with_reuse_disabled() {
    // `reuse_reifier=false` does NOT override explicit `rdf:reifies` reifiers:
    // the user-named reifier identity is always honored.
    let body: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let r_blank = BlankIdBuf::new("_:r".to_owned()).unwrap();
    let r_id = Id::BlankId(r_blank);

    let src = TestGraph(vec![
        Triple(r_id, IriBuf::from(RDF_REIFIES), LocalTerm::triple(body.clone())),
        Triple(s_iri("a"), ex("says"), LocalTerm::triple(body)),
    ]);

    let mut dest = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut dest, &mut g, UnstarOptions { reuse_reifier: false });

    // Cache from rdf:reifies pre-pass forces shared reifier _:r — only 4
    // reification triples + 1 rewritten = 5.
    assert_eq!(dest.0.len(), 5);
    assert_eq!(count_with_predicate(&dest, RDF_TYPE.as_str()), 1);
    let says = dest.0.iter().find(|t| t.1.as_str() == "http://example.org/says").unwrap();
    assert!(matches!(&says.2, LocalTerm::BlankId(b) if b.as_str() == "_:r"));
    let _ = r_id;
}

#[test]
fn unstar_reifies_only_input_emits_four_tuple() {
    // Input has only `_:r rdf:reifies <<( s p o )>>` — no other triples.
    // Output must still contain the four-tuple (data not lost).
    let body: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let r_blank = BlankIdBuf::new("_:r".to_owned()).unwrap();
    let r_id = Id::BlankId(r_blank);

    let src = TestGraph(vec![Triple(r_id, IriBuf::from(RDF_REIFIES), LocalTerm::triple(body))]);

    let mut dest = TestGraph::default();
    let mut g = Blank::new();
    unstar_graph(&src, &mut dest, &mut g, UnstarOptions::default());

    assert_eq!(dest.0.len(), 4);
    assert_eq!(count_with_predicate(&dest, RDF_TYPE.as_str()), 1);
    assert_eq!(count_with_predicate(&dest, RDF_SUBJECT.as_str()), 1);
    assert_eq!(count_with_predicate(&dest, RDF_PREDICATE.as_str()), 1);
    assert_eq!(count_with_predicate(&dest, RDF_OBJECT.as_str()), 1);
    assert_eq!(count_with_predicate(&dest, RDF_REIFIES.as_str()), 0);
}
