#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used, clippy::get_unwrap)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rdf_rs::{
    Quad,
    dataset::{BTreeDataset, HashDataset, IndexedBTreeDataset, IndexedHashDataset, PatternMatchingDataset},
    pattern::CanonicalQuadPattern,
};
use std::hint::black_box;

const SIZES: &[usize] = &[100, 1000, 10_000];

/// N quads of `usize` resources: `(i, P, i+1)` with a fixed predicate `P = 0`.
fn make_quads(n: usize) -> Vec<Quad<usize, usize, usize, usize>> {
    (0..n).map(|i| Quad(i + 1, 0, i + 2, None)).collect()
}

fn make_btree(quads: &[Quad<usize, usize, usize, usize>]) -> BTreeDataset<usize> {
    let mut d = BTreeDataset::new();
    for q in quads {
        d.insert(*q);
    }
    d
}

fn make_hash(quads: &[Quad<usize, usize, usize, usize>]) -> HashDataset<usize> {
    let mut d = HashDataset::new();
    for q in quads {
        d.insert(*q);
    }
    d
}

fn bench_btree_insert(c: &mut Criterion) {
    let mut g = c.benchmark_group("btree_dataset/insert");
    for &n in SIZES {
        let quads = make_quads(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &quads, |b, quads| {
            b.iter(|| {
                let mut d: BTreeDataset<usize> = BTreeDataset::new();
                for q in quads {
                    d.insert(black_box(*q));
                }
                black_box(d)
            });
        });
    }
    g.finish();
}

fn bench_btree_contains(c: &mut Criterion) {
    let mut g = c.benchmark_group("btree_dataset/contains");
    for &n in SIZES {
        let quads = make_quads(n);
        let dataset = make_btree(&quads);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(dataset, quads), |b, (d, quads)| {
            b.iter(|| {
                let mut hits = 0usize;
                for q in quads {
                    if d.contains(black_box(q.as_ref())) {
                        hits += 1;
                    }
                }
                black_box(hits)
            });
        });
    }
    g.finish();
}

fn bench_btree_iter(c: &mut Criterion) {
    let mut g = c.benchmark_group("btree_dataset/iter");
    for &n in SIZES {
        let dataset = make_btree(&make_quads(n));
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &dataset, |b, d| {
            b.iter(|| {
                let mut count = 0usize;
                for q in d.iter() {
                    black_box(q);
                    count += 1;
                }
                black_box(count)
            });
        });
    }
    g.finish();
}

fn bench_indexed_build(c: &mut Criterion) {
    let mut g = c.benchmark_group("indexed_btree_dataset/build");
    for &n in SIZES {
        let quads = make_quads(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &quads, |b, quads| {
            b.iter(|| {
                let mut d: IndexedBTreeDataset<usize> = IndexedBTreeDataset::new();
                for q in quads {
                    d.insert(black_box(*q));
                }
                black_box(d)
            });
        });
    }
    g.finish();
}

fn bench_indexed_pattern_given_subject(c: &mut Criterion) {
    let mut g = c.benchmark_group("indexed_btree_dataset/pattern_given_subject");
    for &n in SIZES {
        let quads = make_quads(n);
        let dataset = make_btree(&quads).into_indexed();
        let subjects: Vec<usize> = quads.iter().map(|q| q.0).collect();
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(dataset, subjects), |b, (d, subjects)| {
            b.iter(|| {
                let mut count = 0usize;
                for s in subjects {
                    let pattern = CanonicalQuadPattern::from_option_quad(Quad(Some(s), None, None, Some(None)));
                    for q in d.quad_pattern_matching(pattern) {
                        black_box(q);
                        count += 1;
                    }
                }
                black_box(count)
            });
        });
    }
    g.finish();
}

fn bench_indexed_contains(c: &mut Criterion) {
    let mut g = c.benchmark_group("indexed_btree_dataset/contains_quad");
    for &n in SIZES {
        let quads = make_quads(n);
        let dataset = make_btree(&quads).into_indexed();
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(dataset, quads), |b, (d, quads)| {
            b.iter(|| {
                let mut hits = 0usize;
                for q in quads {
                    if d.contains_quad(black_box(q.as_ref())) {
                        hits += 1;
                    }
                }
                black_box(hits)
            });
        });
    }
    g.finish();
}

fn bench_hash_insert(c: &mut Criterion) {
    let mut g = c.benchmark_group("hash_dataset/insert");
    for &n in SIZES {
        let quads = make_quads(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &quads, |b, quads| {
            b.iter(|| {
                let mut d: HashDataset<usize> = HashDataset::new();
                for q in quads {
                    d.insert(black_box(*q));
                }
                black_box(d)
            });
        });
    }
    g.finish();
}

fn bench_hash_contains(c: &mut Criterion) {
    let mut g = c.benchmark_group("hash_dataset/contains");
    for &n in SIZES {
        let quads = make_quads(n);
        let dataset = make_hash(&quads);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(dataset, quads), |b, (d, quads)| {
            b.iter(|| {
                let mut hits = 0usize;
                for q in quads {
                    if d.contains(black_box(q.as_ref())) {
                        hits += 1;
                    }
                }
                black_box(hits)
            });
        });
    }
    g.finish();
}

fn bench_hash_iter(c: &mut Criterion) {
    let mut g = c.benchmark_group("hash_dataset/iter");
    for &n in SIZES {
        let dataset = make_hash(&make_quads(n));
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &dataset, |b, d| {
            b.iter(|| {
                let mut count = 0usize;
                for q in d.iter() {
                    black_box(q);
                    count += 1;
                }
                black_box(count)
            });
        });
    }
    g.finish();
}

fn bench_indexed_hash_build(c: &mut Criterion) {
    let mut g = c.benchmark_group("indexed_hash_dataset/build");
    for &n in SIZES {
        let quads = make_quads(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &quads, |b, quads| {
            b.iter(|| {
                let mut d: IndexedHashDataset<usize> = IndexedHashDataset::new();
                for q in quads {
                    d.insert(black_box(*q));
                }
                black_box(d)
            });
        });
    }
    g.finish();
}

fn bench_indexed_hash_pattern_given_subject(c: &mut Criterion) {
    let mut g = c.benchmark_group("indexed_hash_dataset/pattern_given_subject");
    for &n in SIZES {
        let quads = make_quads(n);
        let dataset = make_hash(&quads).into_indexed();
        let subjects: Vec<usize> = quads.iter().map(|q| q.0).collect();
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(dataset, subjects), |b, (d, subjects)| {
            b.iter(|| {
                let mut count = 0usize;
                for s in subjects {
                    let pattern = CanonicalQuadPattern::from_option_quad(Quad(Some(s), None, None, Some(None)));
                    for q in d.quad_pattern_matching(pattern) {
                        black_box(q);
                        count += 1;
                    }
                }
                black_box(count)
            });
        });
    }
    g.finish();
}

fn bench_indexed_hash_contains(c: &mut Criterion) {
    let mut g = c.benchmark_group("indexed_hash_dataset/contains_quad");
    for &n in SIZES {
        let quads = make_quads(n);
        let dataset = make_hash(&quads).into_indexed();
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(dataset, quads), |b, (d, quads)| {
            b.iter(|| {
                let mut hits = 0usize;
                for q in quads {
                    if d.contains_quad(black_box(q.as_ref())) {
                        hits += 1;
                    }
                }
                black_box(hits)
            });
        });
    }
    g.finish();
}

criterion_group!(
    benches,
    bench_btree_insert,
    bench_btree_contains,
    bench_btree_iter,
    bench_indexed_build,
    bench_indexed_pattern_given_subject,
    bench_indexed_contains,
    bench_hash_insert,
    bench_hash_contains,
    bench_hash_iter,
    bench_indexed_hash_build,
    bench_indexed_hash_pattern_given_subject,
    bench_indexed_hash_contains,
);
criterion_main!(benches);
