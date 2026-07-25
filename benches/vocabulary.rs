#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used, clippy::get_unwrap)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use iri_rs::IriBuf;
use rdfx::{
    BlankIdBuf, Literal, LiteralRef,
    vocabulary::{
        BlankIdVocabulary, BlankIdVocabularyMut, IndexVocabulary, IriVocabulary,
        IriVocabularyMut, LiteralVocabulary, LiteralVocabularyMut, Scoped,
    },
};
use std::hint::black_box;

const SIZES: &[usize] = &[100, 1000, 10_000];

fn make_iris(n: usize) -> Vec<IriBuf> {
    (0..n)
        .map(|i| IriBuf::new(format!("http://example.org/resource/{i}")).unwrap())
        .collect()
}

fn make_blank_ids(n: usize) -> Vec<BlankIdBuf> {
    (0..n)
        .map(|i| BlankIdBuf::from_suffix(&format!("b{i}")).unwrap())
        .collect()
}

fn make_literals(n: usize) -> Vec<Literal> {
    (0..n).map(|i| Literal::plain(format!("value-{i}"))).collect()
}

fn fill_iris(v: &mut IndexVocabulary, iris: &[IriBuf]) {
    for iri in iris {
        v.insert(iri.as_ref());
    }
}

fn fill_blanks(v: &mut IndexVocabulary, blanks: &[BlankIdBuf]) {
    for b in blanks {
        v.insert_blank_id(b.as_blank_id_ref());
    }
}

fn fill_literals(v: &mut IndexVocabulary, lits: &[Literal]) {
    for l in lits {
        v.insert_literal(l.as_ref());
    }
}

// ---- IRI ----

fn bench_iri_insert_unique(c: &mut Criterion) {
    let mut g = c.benchmark_group("index_vocabulary/iri/insert_unique");
    for &n in SIZES {
        let iris = make_iris(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &iris, |b, iris| {
            b.iter(|| {
                let mut v: IndexVocabulary = IndexVocabulary::new();
                for iri in iris {
                    black_box(v.insert(black_box(iri.as_ref())));
                }
                black_box(v)
            });
        });
    }
    g.finish();
}

fn bench_iri_insert_repeat(c: &mut Criterion) {
    // Cache-hit path: every insert finds an existing entry. Pre-fill once,
    // then time only the repeated insert calls (state stays constant).
    let mut g = c.benchmark_group("index_vocabulary/iri/insert_repeat");
    for &n in SIZES {
        let iris = make_iris(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &iris, |b, iris| {
            let mut v: IndexVocabulary = IndexVocabulary::new();
            fill_iris(&mut v, iris);
            b.iter(|| {
                for iri in iris {
                    black_box(v.insert(black_box(iri.as_ref())));
                }
            });
        });
    }
    g.finish();
}

fn bench_iri_get_hit(c: &mut Criterion) {
    let mut g = c.benchmark_group("index_vocabulary/iri/get_hit");
    for &n in SIZES {
        let iris = make_iris(n);
        let mut v: IndexVocabulary = IndexVocabulary::new();
        fill_iris(&mut v, &iris);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &(v, iris), |b, (v, iris)| {
            b.iter(|| {
                let mut hits = 0usize;
                for iri in iris {
                    if v.get(black_box(iri.as_ref())).is_some() {
                        hits += 1;
                    }
                }
                black_box(hits)
            });
        });
    }
    g.finish();
}

fn bench_iri_get_miss(c: &mut Criterion) {
    let mut g = c.benchmark_group("index_vocabulary/iri/get_miss");
    for &n in SIZES {
        let iris = make_iris(n);
        let misses: Vec<IriBuf> = (0..n)
            .map(|i| IriBuf::new(format!("http://example.org/missing/{i}")).unwrap())
            .collect();
        let mut v: IndexVocabulary = IndexVocabulary::new();
        fill_iris(&mut v, &iris);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(
            BenchmarkId::from_parameter(n),
            &(v, misses),
            |b, (v, misses)| {
                b.iter(|| {
                    let mut hits = 0usize;
                    for iri in misses {
                        if v.get(black_box(iri.as_ref())).is_some() {
                            hits += 1;
                        }
                    }
                    black_box(hits)
                });
            },
        );
    }
    g.finish();
}

// ---- BlankId ----

fn bench_blank_insert_unique(c: &mut Criterion) {
    let mut g = c.benchmark_group("index_vocabulary/blank/insert_unique");
    for &n in SIZES {
        let blanks = make_blank_ids(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &blanks, |b, blanks| {
            b.iter(|| {
                let mut v: IndexVocabulary = IndexVocabulary::new();
                for x in blanks {
                    black_box(v.insert_blank_id(black_box(x.as_blank_id_ref())));
                }
                black_box(v)
            });
        });
    }
    g.finish();
}

fn bench_blank_insert_repeat(c: &mut Criterion) {
    let mut g = c.benchmark_group("index_vocabulary/blank/insert_repeat");
    for &n in SIZES {
        let blanks = make_blank_ids(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &blanks, |b, blanks| {
            let mut v: IndexVocabulary = IndexVocabulary::new();
            fill_blanks(&mut v, blanks);
            b.iter(|| {
                for x in blanks {
                    black_box(v.insert_blank_id(black_box(x.as_blank_id_ref())));
                }
            });
        });
    }
    g.finish();
}

fn bench_blank_get_hit(c: &mut Criterion) {
    let mut g = c.benchmark_group("index_vocabulary/blank/get_hit");
    for &n in SIZES {
        let blanks = make_blank_ids(n);
        let mut v: IndexVocabulary = IndexVocabulary::new();
        fill_blanks(&mut v, &blanks);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(
            BenchmarkId::from_parameter(n),
            &(v, blanks),
            |b, (v, blanks)| {
                b.iter(|| {
                    let mut hits = 0usize;
                    for x in blanks {
                        if v.get_blank_id(black_box(x.as_blank_id_ref())).is_some() {
                            hits += 1;
                        }
                    }
                    black_box(hits)
                });
            },
        );
    }
    g.finish();
}

// ---- Literal ----

fn bench_literal_insert_unique(c: &mut Criterion) {
    let mut g = c.benchmark_group("index_vocabulary/literal/insert_unique");
    for &n in SIZES {
        let lits = make_literals(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &lits, |b, lits| {
            b.iter(|| {
                let mut v: IndexVocabulary = IndexVocabulary::new();
                for l in lits {
                    black_box(v.insert_literal(black_box(l.as_ref())));
                }
                black_box(v)
            });
        });
    }
    g.finish();
}

fn bench_literal_insert_repeat(c: &mut Criterion) {
    let mut g = c.benchmark_group("index_vocabulary/literal/insert_repeat");
    for &n in SIZES {
        let lits = make_literals(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &lits, |b, lits| {
            let mut v: IndexVocabulary = IndexVocabulary::new();
            fill_literals(&mut v, lits);
            b.iter(|| {
                for l in lits {
                    black_box(v.insert_literal(black_box(l.as_ref())));
                }
            });
        });
    }
    g.finish();
}

fn bench_literal_get_hit(c: &mut Criterion) {
    let mut g = c.benchmark_group("index_vocabulary/literal/get_hit");
    for &n in SIZES {
        let lits = make_literals(n);
        let mut v: IndexVocabulary = IndexVocabulary::new();
        fill_literals(&mut v, &lits);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(
            BenchmarkId::from_parameter(n),
            &(v, lits),
            |b, (v, lits)| {
                b.iter(|| {
                    let mut hits = 0usize;
                    for l in lits {
                        let r: LiteralRef<'_> = l.as_ref();
                        if v.get_literal(black_box(r)).is_some() {
                            hits += 1;
                        }
                    }
                    black_box(hits)
                });
            },
        );
    }
    g.finish();
}

fn bench_literal_get_miss(c: &mut Criterion) {
    let mut g = c.benchmark_group("index_vocabulary/literal/get_miss");
    for &n in SIZES {
        let lits = make_literals(n);
        let misses: Vec<Literal> = (0..n).map(|i| Literal::plain(format!("missing-{i}"))).collect();
        let mut v: IndexVocabulary = IndexVocabulary::new();
        fill_literals(&mut v, &lits);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(
            BenchmarkId::from_parameter(n),
            &(v, misses),
            |b, (v, misses)| {
                b.iter(|| {
                    let mut hits = 0usize;
                    for l in misses {
                        if v.get_literal(black_box(l.as_ref())).is_some() {
                            hits += 1;
                        }
                    }
                    black_box(hits)
                });
            },
        );
    }
    g.finish();
}

// ---- Mixed RDF-parse-like workload ----

fn bench_mixed_workload(c: &mut Criterion) {
    // ~70% IRI / 20% literal / 10% blank, with heavy repetition (predicates).
    let mut g = c.benchmark_group("index_vocabulary/mixed_workload");
    for &n in SIZES {
        let predicates = make_iris(8);
        let subjects = make_iris(n / 2);
        let blanks = make_blank_ids(n / 10 + 1);
        let lits = make_literals(n / 5 + 1);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(
            BenchmarkId::from_parameter(n),
            &(predicates, subjects, blanks, lits),
            |bb, (preds, subs, bl, lits)| {
                bb.iter(|| {
                    let mut v: IndexVocabulary = IndexVocabulary::new();
                    for i in 0..n {
                        // subject (mostly IRI, sometimes blank)
                        if i % 10 == 0 {
                            black_box(v.insert_blank_id(bl[i % bl.len()].as_blank_id_ref()));
                        } else {
                            black_box(v.insert(subs[i % subs.len()].as_ref()));
                        }
                        // predicate (small set, hot cache hits)
                        black_box(v.insert(preds[i % preds.len()].as_ref()));
                        // object (literal sometimes, IRI otherwise)
                        if i % 5 == 0 {
                            black_box(v.insert_literal(lits[i % lits.len()].as_ref()));
                        } else {
                            black_box(v.insert(subs[(i + 1) % subs.len()].as_ref()));
                        }
                    }
                    black_box(v)
                });
            },
        );
    }
    g.finish();
}

// ---- Scoped wrapper hot path ----

fn bench_scoped_blank_repeat(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoped/blank/insert_repeat");
    for &n in SIZES {
        let blanks = make_blank_ids(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &blanks, |b, blanks| {
            b.iter(|| {
                let mut inner: IndexVocabulary = IndexVocabulary::new();
                let mut s = Scoped::new(&mut inner, "g0");
                for x in blanks {
                    black_box(s.insert_blank_id(black_box(x.as_blank_id_ref())));
                }
                // second pass: cache-hit path through Scoped's local map.
                for x in blanks {
                    black_box(s.insert_blank_id(black_box(x.as_blank_id_ref())));
                }
                black_box(())
            });
        });
    }
    g.finish();
}

criterion_group!(
    benches,
    bench_iri_insert_unique,
    bench_iri_insert_repeat,
    bench_iri_get_hit,
    bench_iri_get_miss,
    bench_blank_insert_unique,
    bench_blank_insert_repeat,
    bench_blank_get_hit,
    bench_literal_insert_unique,
    bench_literal_insert_repeat,
    bench_literal_get_hit,
    bench_literal_get_miss,
    bench_mixed_workload,
    bench_scoped_blank_repeat,
);
criterion_main!(benches);
