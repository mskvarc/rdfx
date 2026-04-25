use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rdf_term::{
    BlankIdBuf,
    LocalTerm,
    Quad,
    dataset::{
        BTreeDataset,
        HashDataset,
        isomorphism::{find_bijection, find_hash_bijection},
    },
    iri,
};
use std::hint::black_box;

const SIZES: &[usize] = &[32, 128, 512];

fn iri_term(i: usize) -> LocalTerm {
    LocalTerm::iri(iri_rs::IriBuf::new(format!("http://example.org/r/{i}")).unwrap())
}

fn blank_term(label: &str) -> LocalTerm {
    LocalTerm::Anonymous(BlankIdBuf::new(format!("_:{label}")).unwrap())
}

/// Build a blank-chain dataset of `n` quads.
///
/// `<_:b{k}> <p> <_:b{k+1}>` for `k in 0..n`, where blank labels use
/// `prefix` so two datasets can be built with distinct blank labels
/// but identical structure (isomorphic).
fn blank_chain(n: usize, prefix: &str) -> BTreeDataset {
    let predicate = LocalTerm::iri(iri_rs::IriBuf::from(iri!("http://example.org/p")));
    let mut d = BTreeDataset::new();
    for k in 0..n {
        let s = blank_term(&format!("{prefix}{k}"));
        let o = blank_term(&format!("{prefix}{}", k + 1));
        d.insert(Quad(s, predicate.clone(), o, None));
    }
    d
}

/// Fully named chain — no blank nodes; isomorphism should short-circuit fast.
fn iri_chain(n: usize) -> BTreeDataset {
    let predicate = LocalTerm::iri(iri_rs::IriBuf::from(iri!("http://example.org/p")));
    let mut d = BTreeDataset::new();
    for k in 0..n {
        d.insert(Quad(iri_term(k), predicate.clone(), iri_term(k + 1), None));
    }
    d
}

fn blank_chain_hash(n: usize, prefix: &str) -> HashDataset {
    let predicate = LocalTerm::iri(iri_rs::IriBuf::from(iri!("http://example.org/p")));
    let mut d = HashDataset::new();
    for k in 0..n {
        let s = blank_term(&format!("{prefix}{k}"));
        let o = blank_term(&format!("{prefix}{}", k + 1));
        d.insert(Quad(s, predicate.clone(), o, None));
    }
    d
}

fn iri_chain_hash(n: usize) -> HashDataset {
    let predicate = LocalTerm::iri(iri_rs::IriBuf::from(iri!("http://example.org/p")));
    let mut d = HashDataset::new();
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
            bn.iter(|| black_box(find_bijection(black_box(a), black_box(b))));
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
            bn.iter(|| black_box(find_bijection(black_box(a), black_box(b))));
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
            bn.iter(|| black_box(find_hash_bijection(black_box(a), black_box(b))));
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
            bn.iter(|| black_box(find_hash_bijection(black_box(a), black_box(b))));
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
