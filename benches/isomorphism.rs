#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used, clippy::get_unwrap)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use iri_rs::{Iri, IriBuf};
use rdfx::{
    BlankId,
    BlankIdBuf,
    CowLiteral,
    LiteralRef,
    Quad,
    dataset::{BTreeDataset, HashDataset, isomorphism::find_bijection_with},
    impl_resource,
    interpretation::{Interpretation, LocalInterpretation, ReverseInterpretation, ReverseLocalInterpretation},
    iri,
};
use std::{borrow::Cow, hint::black_box};

const SIZES: &[usize] = &[32, 128, 512];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BenchId {
    Iri(IriBuf),
    Blank(BlankIdBuf),
}

impl_resource!(BenchId);

pub struct BenchInterp;

impl Interpretation for BenchInterp {
    type Resource = BenchId;
    fn iri(&self, iri: Iri<&str>) -> Option<BenchId> {
        Some(BenchId::Iri(iri.into()))
    }
    fn literal<'a>(&self, _literal: impl Into<LiteralRef<'a>>) -> Option<BenchId> {
        None
    }
}

impl LocalInterpretation for BenchInterp {
    fn blank_id(&self, blank_id: &BlankId) -> Option<BenchId> {
        Some(BenchId::Blank(blank_id.to_owned()))
    }
}

impl ReverseInterpretation for BenchInterp {
    type Iris<'a> = std::option::IntoIter<Cow<'a, IriBuf>>;
    type Literals<'a> = std::option::IntoIter<CowLiteral<'a>>;
    fn iris_of<'a>(&'a self, r: &'a Self::Resource) -> Self::Iris<'a> {
        match r {
            BenchId::Iri(i) => Some(Cow::Borrowed(i)).into_iter(),
            _ => None.into_iter(),
        }
    }
    fn literals_of<'a>(&'a self, _r: &'a Self::Resource) -> Self::Literals<'a> {
        None.into_iter()
    }
}

impl ReverseLocalInterpretation for BenchInterp {
    type BlankIds<'a> = std::option::IntoIter<Cow<'a, BlankId>>;
    fn blank_ids_of<'a>(&'a self, r: &'a Self::Resource) -> Self::BlankIds<'a> {
        match r {
            BenchId::Blank(b) => Some(Cow::Borrowed(b.as_blank_id_ref())).into_iter(),
            _ => None.into_iter(),
        }
    }
}

fn iri_term(i: usize) -> BenchId {
    BenchId::Iri(IriBuf::new(format!("http://example.org/r/{i}")).unwrap())
}

fn blank_term(label: &str) -> BenchId {
    BenchId::Blank(BlankIdBuf::new(format!("_:{label}")).unwrap())
}

/// Build a blank-chain dataset of `n` quads.
///
/// `<_:b{k}> <p> <_:b{k+1}>` for `k in 0..n`, where blank labels use
/// `prefix` so two datasets can be built with distinct blank labels
/// but identical structure (isomorphic).
fn blank_chain(n: usize, prefix: &str) -> BTreeDataset<BenchId> {
    let predicate = BenchId::Iri(IriBuf::from(iri!("http://example.org/p")));
    let mut d: BTreeDataset<BenchId> = BTreeDataset::new();
    for k in 0..n {
        let s = blank_term(&format!("{prefix}{k}"));
        let o = blank_term(&format!("{prefix}{}", k + 1));
        d.insert(Quad(s, predicate.clone(), o, None));
    }
    d
}

/// Fully named chain — no blank nodes; isomorphism should short-circuit fast.
fn iri_chain(n: usize) -> BTreeDataset<BenchId> {
    let predicate = BenchId::Iri(IriBuf::from(iri!("http://example.org/p")));
    let mut d: BTreeDataset<BenchId> = BTreeDataset::new();
    for k in 0..n {
        d.insert(Quad(iri_term(k), predicate.clone(), iri_term(k + 1), None));
    }
    d
}

fn blank_chain_hash(n: usize, prefix: &str) -> HashDataset<BenchId> {
    let predicate = BenchId::Iri(IriBuf::from(iri!("http://example.org/p")));
    let mut d: HashDataset<BenchId> = HashDataset::new();
    for k in 0..n {
        let s = blank_term(&format!("{prefix}{k}"));
        let o = blank_term(&format!("{prefix}{}", k + 1));
        d.insert(Quad(s, predicate.clone(), o, None));
    }
    d
}

fn iri_chain_hash(n: usize) -> HashDataset<BenchId> {
    let predicate = BenchId::Iri(IriBuf::from(iri!("http://example.org/p")));
    let mut d: HashDataset<BenchId> = HashDataset::new();
    for k in 0..n {
        d.insert(Quad(iri_term(k), predicate.clone(), iri_term(k + 1), None));
    }
    d
}

fn bench_iso_named(c: &mut Criterion) {
    let mut g = c.benchmark_group("isomorphism/named_chain");
    for &n in SIZES {
        let a = iri_chain(n);
        let b = iri_chain(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(a, b), |bn, (a, b)| {
            bn.iter(|| black_box(find_bijection_with(&BenchInterp, black_box(a), black_box(b))));
        });
    }
    g.finish();
}

fn bench_iso_blank_relabeled(c: &mut Criterion) {
    let mut g = c.benchmark_group("isomorphism/blank_chain_relabeled");
    for &n in SIZES {
        let a = blank_chain(n, "a");
        let b = blank_chain(n, "b");
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(a, b), |bn, (a, b)| {
            bn.iter(|| black_box(find_bijection_with(&BenchInterp, black_box(a), black_box(b))));
        });
    }
    g.finish();
}

fn bench_iso_hash_named(c: &mut Criterion) {
    let mut g = c.benchmark_group("isomorphism_hash/named_chain");
    for &n in SIZES {
        let a = iri_chain_hash(n);
        let b = iri_chain_hash(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(a, b), |bn, (a, b)| {
            bn.iter(|| black_box(find_bijection_with(&BenchInterp, black_box(a), black_box(b))));
        });
    }
    g.finish();
}

fn bench_iso_hash_blank_relabeled(c: &mut Criterion) {
    let mut g = c.benchmark_group("isomorphism_hash/blank_chain_relabeled");
    for &n in SIZES {
        let a = blank_chain_hash(n, "a");
        let b = blank_chain_hash(n, "b");
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(a, b), |bn, (a, b)| {
            bn.iter(|| black_box(find_bijection_with(&BenchInterp, black_box(a), black_box(b))));
        });
    }
    g.finish();
}

criterion_group!(
    benches,
    bench_iso_named,
    bench_iso_blank_relabeled,
    bench_iso_hash_named,
    bench_iso_hash_blank_relabeled,
);
criterion_main!(benches);
