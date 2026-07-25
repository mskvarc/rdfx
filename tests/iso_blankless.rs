//! Regression tests for blank-free quad comparison in
//! [`dataset_equivalent_with`] / [`graph_equivalent_with`].
//!
//! Earlier the bijection algorithm only walked blank-touching quads; two
//! datasets with disjoint IRI-only quads were reported as equivalent.
#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use iri_rs::Iri;
use rdfx::{
    BlankId,
    BlankIdBuf,
    CowLiteral,
    IriBuf,
    LiteralRef,
    Quad,
    Triple,
    dataset::{
        BTreeDataset,
        HashGraph,
        isomorphism::{dataset_equivalent_with, graph_equivalent_with},
    },
    impl_resource,
    interpretation::{Interpretation, LocalInterpretation, ReverseInterpretation, ReverseLocalInterpretation},
};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Id {
    Iri(IriBuf),
    Blank(BlankIdBuf),
}

impl_resource!(Id);

struct Interp;

impl Interpretation for Interp {
    type Resource = Id;
    fn iri(&self, iri: Iri<&str>) -> Option<Id> {
        Some(Id::Iri(iri.into()))
    }
    fn literal<'a>(&self, _l: impl Into<LiteralRef<'a>>) -> Option<Id> {
        None
    }
}

impl LocalInterpretation for Interp {
    fn blank_id(&self, b: &BlankId) -> Option<Id> {
        Some(Id::Blank(b.to_owned()))
    }
}

impl ReverseInterpretation for Interp {
    type Iris<'a> = std::option::IntoIter<Cow<'a, IriBuf>>;
    type Literals<'a> = std::option::IntoIter<CowLiteral<'a>>;
    fn iris_of<'a>(&'a self, r: &'a Id) -> Self::Iris<'a> {
        match r {
            Id::Iri(i) => Some(Cow::Borrowed(i)).into_iter(),
            _ => None.into_iter(),
        }
    }
    fn literals_of<'a>(&'a self, _r: &'a Id) -> Self::Literals<'a> {
        None.into_iter()
    }
}

impl ReverseLocalInterpretation for Interp {
    type BlankIds<'a> = std::option::IntoIter<Cow<'a, BlankId>>;
    fn blank_ids_of<'a>(&'a self, r: &'a Id) -> Self::BlankIds<'a> {
        match r {
            Id::Blank(b) => Some(Cow::Borrowed(b.as_blank_id_ref())).into_iter(),
            _ => None.into_iter(),
        }
    }
}

fn iri(s: &'static str) -> Id {
    Id::Iri(iri_rs::IriBuf::new(s.to_owned()).unwrap())
}

fn blank(s: &'static str) -> Id {
    Id::Blank(BlankIdBuf::from_suffix(s).unwrap())
}

#[test]
fn blankless_distinct_iri_subjects_not_equivalent() {
    let p = iri("http://ex/p");
    let o = iri("http://ex/o");

    let mut a = BTreeDataset::<Id>::new();
    a.insert(Quad::new(iri("http://ex/s1"), p.clone(), o.clone(), None));
    let mut b = BTreeDataset::<Id>::new();
    b.insert(Quad::new(iri("http://ex/s2"), p, o, None));

    assert!(!dataset_equivalent_with(&Interp, &a, &b));
}

#[test]
fn blankless_identical_singleton_equivalent() {
    let q = || Quad::new(iri("http://ex/s"), iri("http://ex/p"), iri("http://ex/o"), None);

    let mut a = BTreeDataset::<Id>::new();
    a.insert(q());
    let mut b = BTreeDataset::<Id>::new();
    b.insert(q());

    assert!(dataset_equivalent_with(&Interp, &a, &b));
}

#[test]
fn mixed_blank_matches_but_blankless_differs_not_equivalent() {
    // Both sides have one blank-touching quad that bijects, but the
    // blankless quads differ. The pre-fix algorithm only walked blank
    // signatures and missed this.
    let p = iri("http://ex/p");
    let o = iri("http://ex/o");

    let mut a = BTreeDataset::<Id>::new();
    a.insert(Quad::new(blank("a0"), p.clone(), o.clone(), None));
    a.insert(Quad::new(iri("http://ex/s"), p.clone(), iri("http://ex/o1"), None));

    let mut b = BTreeDataset::<Id>::new();
    b.insert(Quad::new(blank("x0"), p.clone(), o, None));
    b.insert(Quad::new(iri("http://ex/s"), p, iri("http://ex/o2"), None));

    assert!(!dataset_equivalent_with(&Interp, &a, &b));
}

#[test]
fn graph_blankless_distinct_iri_subjects_not_equivalent() {
    let p = iri("http://ex/p");
    let o = iri("http://ex/o");

    let mut a = HashGraph::<Id>::new();
    a.insert(Triple::new(iri("http://ex/s1"), p.clone(), o.clone()));
    let mut b = HashGraph::<Id>::new();
    b.insert(Triple::new(iri("http://ex/s2"), p, o));

    assert!(!graph_equivalent_with(&Interp, &a, &b));
}
