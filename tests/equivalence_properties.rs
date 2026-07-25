//! Property tests for graph/dataset equivalence (RDF 1.1 §3.6 / §4.4).
//!
//! Reflexivity, symmetry, sample transitivity, and rejection of size /
//! structure mismatches over the structural-equivalence entry points
//! (`graph_equivalent` / `dataset_equivalent`, which use `BlindInterpretation`).
#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use rdfx::{
    Quad,
    Triple,
    dataset::{
        BTreeDataset,
        BTreeGraph,
        isomorphism::{dataset_equivalent, graph_equivalent},
    },
};

fn graph_with(triples: &[(usize, usize, usize)]) -> BTreeGraph<usize> {
    let mut g: BTreeGraph<usize> = BTreeGraph::new();
    for &(s, p, o) in triples {
        g.insert(Triple(s, p, o));
    }
    g
}

fn dataset_with(quads: &[(usize, usize, usize, Option<usize>)]) -> BTreeDataset<usize> {
    let mut d: BTreeDataset<usize> = BTreeDataset::new();
    for &(s, p, o, g) in quads {
        d.insert(Quad(s, p, o, g));
    }
    d
}

// ----- reflexivity -----

#[test]
fn graph_equivalence_is_reflexive() {
    let g = graph_with(&[(1, 0, 2), (2, 0, 3), (3, 0, 4)]);
    assert!(graph_equivalent(&g, &g));
}

#[test]
fn dataset_equivalence_is_reflexive() {
    let d = dataset_with(&[(1, 0, 2, None), (3, 0, 4, Some(7)), (5, 0, 6, None)]);
    assert!(dataset_equivalent(&d, &d));
}

#[test]
fn empty_graphs_are_equivalent() {
    let a: BTreeGraph<usize> = BTreeGraph::new();
    let b: BTreeGraph<usize> = BTreeGraph::new();
    assert!(graph_equivalent(&a, &b));
}

#[test]
fn empty_datasets_are_equivalent() {
    let a: BTreeDataset<usize> = BTreeDataset::new();
    let b: BTreeDataset<usize> = BTreeDataset::new();
    assert!(dataset_equivalent(&a, &b));
}

// ----- symmetry -----

#[test]
fn graph_equivalence_is_symmetric() {
    let a = graph_with(&[(1, 0, 2), (2, 0, 3)]);
    let b = graph_with(&[(10, 0, 20), (20, 0, 30)]);
    assert_eq!(graph_equivalent(&a, &b), graph_equivalent(&b, &a));
}

#[test]
fn dataset_equivalence_is_symmetric() {
    let a = dataset_with(&[(1, 0, 2, None), (2, 0, 3, Some(7))]);
    let b = dataset_with(&[(10, 0, 20, None), (20, 0, 30, Some(70))]);
    assert_eq!(dataset_equivalent(&a, &b), dataset_equivalent(&b, &a));
}

// ----- sample transitivity -----

#[test]
fn graph_equivalence_transitive_on_relabelings() {
    // Three structurally identical chain graphs with different label sets.
    let a = graph_with(&[(1, 0, 2), (2, 0, 3)]);
    let b = graph_with(&[(10, 0, 20), (20, 0, 30)]);
    let c = graph_with(&[(100, 0, 200), (200, 0, 300)]);
    assert!(graph_equivalent(&a, &b));
    assert!(graph_equivalent(&b, &c));
    assert!(graph_equivalent(&a, &c));
}

#[test]
fn dataset_equivalence_transitive_on_relabelings() {
    let a = dataset_with(&[(1, 0, 2, None), (2, 0, 3, None)]);
    let b = dataset_with(&[(10, 0, 20, None), (20, 0, 30, None)]);
    let c = dataset_with(&[(100, 0, 200, None), (200, 0, 300, None)]);
    assert!(dataset_equivalent(&a, &b));
    assert!(dataset_equivalent(&b, &c));
    assert!(dataset_equivalent(&a, &c));
}

// ----- rejection: size mismatch -----

#[test]
fn graphs_of_different_size_are_not_equivalent() {
    let a = graph_with(&[(1, 0, 2)]);
    let b = graph_with(&[(1, 0, 2), (2, 0, 3)]);
    assert!(!graph_equivalent(&a, &b));
}

#[test]
fn datasets_of_different_size_are_not_equivalent() {
    let a = dataset_with(&[(1, 0, 2, None)]);
    let b = dataset_with(&[(1, 0, 2, None), (2, 0, 3, None)]);
    assert!(!dataset_equivalent(&a, &b));
}

// ----- rejection: structural mismatch -----

#[test]
fn graphs_of_same_size_but_different_structure_are_not_equivalent() {
    // a: chain 1 -> 2 -> 3 (two distinct subjects)
    let a = graph_with(&[(1, 0, 2), (2, 0, 3)]);
    // b: 1 -> 2 and 1 -> 3 (one subject, two distinct objects)
    let b = graph_with(&[(1, 0, 2), (1, 0, 3)]);
    assert!(!graph_equivalent(&a, &b));
}

#[test]
fn datasets_with_different_named_graph_topology_are_not_equivalent() {
    // a has a triple in named graph, b has it in default graph.
    let a = dataset_with(&[(1, 0, 2, Some(7))]);
    let b = dataset_with(&[(1, 0, 2, None)]);
    assert!(!dataset_equivalent(&a, &b));
}
