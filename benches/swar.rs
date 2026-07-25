#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used, clippy::get_unwrap)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use iri_rs::IriRef;
use rdfx::{BlankId, RdfDisplay};
use std::hint::black_box;

fn bench_blankid(c: &mut Criterion) {
    let mut g = c.benchmark_group("blankid/check");

    for s in ["_:b1", "_:b42", "_:b9999"] {
        g.throughput(Throughput::Bytes(s.len() as u64));
        g.bench_with_input(BenchmarkId::new("realistic_short", s.len()), &s, |b, s| {
            b.iter(|| black_box(BlankId::new(black_box(*s)).is_ok()));
        });
    }

    {
        let s = "_:b550e8400-e29b-41d4-a716-446655440000";
        g.throughput(Throughput::Bytes(s.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter("realistic_uuid"), &s, |b, s| {
            b.iter(|| black_box(BlankId::new(black_box(*s)).is_ok()));
        });
    }

    for &n in &[16usize, 64, 256, 1024] {
        let s = format!("_:{}", "a".repeat(n));
        g.throughput(Throughput::Bytes(s.len() as u64));
        g.bench_with_input(BenchmarkId::new("long_ascii_clean", n), &s, |b, s| {
            b.iter(|| black_box(BlankId::new(black_box(s.as_str())).is_ok()));
        });
    }

    for &n in &[16usize, 64, 256, 1024] {
        let s = format!("_:{}.", "a".repeat(n));
        g.throughput(Throughput::Bytes(s.len() as u64));
        g.bench_with_input(BenchmarkId::new("long_ascii_invalid_trailing_dot", n), &s, |b, s| {
            b.iter(|| black_box(BlankId::new(black_box(s.as_str())).is_ok()));
        });
    }

    for &reps in &[3usize, 11, 43, 171] {
        let body: String = "αβγ".repeat(reps);
        let s = format!("_:{body}");
        g.throughput(Throughput::Bytes(s.len() as u64));
        g.bench_with_input(BenchmarkId::new("long_unicode_mixed", s.len()), &s, |b, s| {
            b.iter(|| black_box(BlankId::new(black_box(s.as_str())).is_ok()));
        });
    }

    {
        let s = format!("_:é{}", "a".repeat(1024));
        g.throughput(Throughput::Bytes(s.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter("adversarial_high_bit_at_start"), &s, |b, s| {
            b.iter(|| black_box(BlankId::new(black_box(s.as_str())).is_ok()));
        });
    }

    g.finish();
}

fn bench_iri(c: &mut Criterion) {
    let mut g = c.benchmark_group("iri/rdf_fmt");

    let clean_short = [
        "http://example.org/r/1",
        "http://example.org/r/longer-path/segment",
        "https://www.w3.org/2001/XMLSchema#string",
        "http://xmlns.com/foaf/0.1/Person",
        "https://example.org/api/v2/things/abc-123-def",
    ];
    for (i, s) in clean_short.iter().enumerate() {
        let iri = IriRef::parse_unchecked(s.to_string());
        g.throughput(Throughput::Bytes(s.len() as u64));
        g.bench_with_input(BenchmarkId::new("clean_short", i), &iri, |b, iri| {
            b.iter(|| black_box(format!("{}", black_box(iri).rdf_display())));
        });
    }

    for &n in &[64usize, 256, 1024, 8192] {
        let s = format!("http://example.org/{}", "a".repeat(n));
        let iri = IriRef::parse_unchecked(s.clone());
        g.throughput(Throughput::Bytes(s.len() as u64));
        g.bench_with_input(BenchmarkId::new("clean_long", n), &iri, |b, iri| {
            b.iter(|| black_box(format!("{}", black_box(iri).rdf_display())));
        });
    }

    for &n in &[256usize, 1024, 8192] {
        let chunks = n / 256;
        let mut s = String::with_capacity(n);
        for _ in 0..chunks {
            s.push_str(&"a".repeat(255));
            s.push('<');
        }
        let iri = IriRef::parse_unchecked(s.clone());
        g.throughput(Throughput::Bytes(s.len() as u64));
        g.bench_with_input(BenchmarkId::new("escapes_sparse", n), &iri, |b, iri| {
            b.iter(|| black_box(format!("{}", black_box(iri).rdf_display())));
        });
    }

    for &n in &[256usize, 1024] {
        let mut s = String::with_capacity(n);
        for _ in 0..(n / 2) {
            s.push('a');
            s.push('<');
        }
        let iri = IriRef::parse_unchecked(s.clone());
        g.throughput(Throughput::Bytes(s.len() as u64));
        g.bench_with_input(BenchmarkId::new("escapes_dense", n), &iri, |b, iri| {
            b.iter(|| black_box(format!("{}", black_box(iri).rdf_display())));
        });
    }

    {
        let s = "http://пример.рф/path/中文/text".to_string();
        let len = s.len();
        let iri = IriRef::parse_unchecked(s);
        g.throughput(Throughput::Bytes(len as u64));
        g.bench_with_input(BenchmarkId::from_parameter("non_ascii"), &iri, |b, iri| {
            b.iter(|| black_box(format!("{}", black_box(iri).rdf_display())));
        });
    }

    g.finish();
}

criterion_group!(benches, bench_blankid, bench_iri);
criterion_main!(benches);
