//! Benchmarks for graph isomorphism with RDF 1.2 triple terms.
//!
//! Exercises [`graph_equivalent_with`] over graphs whose triples carry
//! triple-term objects at depth 1, depth 2, and depth 3. Mirror style of
//! [`benches/isomorphism.rs`]. Signal-only — no numeric assertions.
#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use std::borrow::Cow;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use iri_rs::IriBuf;
use rdfx::{
    CowLiteral,
    Id,
    Interpretation,
    LiteralRef,
    LocalTerm,
    Term,
    Triple,
    TripleTerm,
    dataset::{BTreeGraph, isomorphism::graph_equivalent_with},
    impl_resource,
    interpretation::ReverseInterpretation,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct Lex(LocalTerm);

impl_resource!(Lex);

impl Lex {
    fn from_local_term_ref(t: &LocalTerm) -> &Self {
        // Safety: `Lex` is `#[repr(transparent)]` over `LocalTerm`.
        unsafe { &*((t as *const LocalTerm).cast::<Self>()) }
    }
}

impl rdfx::RdfDisplay for Lex {
    fn rdf_fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.rdf_fmt(f)
    }
}

struct LexInterp;

impl Interpretation for LexInterp {
    type Resource = Lex;
    fn iri(&self, iri: iri_rs::Iri<&str>) -> Option<Lex> {
        Some(Lex(LocalTerm::iri(iri.into())))
    }
    fn literal<'a>(&self, literal: impl Into<LiteralRef<'a>>) -> Option<Lex> {
        Some(Lex(LocalTerm::literal(literal.into().to_owned())))
    }
}

impl ReverseInterpretation for LexInterp {
    type Iris<'a> = std::option::IntoIter<Cow<'a, IriBuf>>;
    type Literals<'a> = std::option::IntoIter<CowLiteral<'a>>;

    fn iris_of<'a>(&'a self, resource: &'a Lex) -> Self::Iris<'a> {
        match &resource.0 {
            LocalTerm::Named(Term::Iri(iri)) => Some(Cow::Borrowed(iri)).into_iter(),
            _ => None.into_iter(),
        }
    }
    fn literals_of<'a>(&'a self, resource: &'a Lex) -> Self::Literals<'a> {
        match &resource.0 {
            LocalTerm::Named(Term::Literal(l)) => Some(l.as_cow()).into_iter(),
            _ => None.into_iter(),
        }
    }
    fn triple_term_components(&self, resource: &Lex) -> Option<(Lex, Lex, Lex)> {
        match &resource.0 {
            LocalTerm::Triple(t) => {
                let s = match &t.0 {
                    Id::Iri(iri) => Lex(LocalTerm::iri(iri.clone())),
                    Id::BlankId(b) => Lex(LocalTerm::BlankId(b.clone())),
                };
                let p = Lex(LocalTerm::iri(t.1.clone()));
                let o = Lex(t.2.clone());
                Some((s, p, o))
            }
            _ => None,
        }
    }

    fn triple_term_components_view<'a>(&'a self, resource: &'a Lex) -> Option<(Cow<'a, Lex>, Cow<'a, Lex>, Cow<'a, Lex>)> {
        match &resource.0 {
            LocalTerm::Triple(t) => {
                let s = match &t.0 {
                    Id::Iri(iri) => Lex(LocalTerm::iri(iri.clone())),
                    Id::BlankId(b) => Lex(LocalTerm::BlankId(b.clone())),
                };
                Some((Cow::Owned(s), Cow::Owned(Lex(LocalTerm::iri(t.1.clone()))), Cow::Borrowed(Lex::from_local_term_ref(&t.2))))
            }
            _ => None,
        }
    }
}

fn iri(s: &str) -> IriBuf {
    IriBuf::new(format!("http://example.org/{s}")).unwrap()
}

/// Build a body of nesting depth `d`. Innermost body uses `i`-indexed IRIs.
fn body(i: usize, d: usize) -> TripleTerm {
    if d == 0 {
        Triple(Id::iri(iri(&format!("s{i}"))), iri(&format!("p{i}")), LocalTerm::iri(iri(&format!("o{i}"))))
    } else {
        let inner = body(i, d - 1);
        Triple(Id::iri(iri(&format!("s{i}d{d}"))), iri(&format!("p{i}d{d}")), LocalTerm::triple(inner))
    }
}

fn make_graph(n: usize, depth: usize) -> BTreeGraph<Lex> {
    let mut g: BTreeGraph<Lex> = BTreeGraph::new();
    for i in 0..n {
        let s = Lex(LocalTerm::iri(iri(&format!("a{i}"))));
        let p = Lex(LocalTerm::iri(iri(&format!("says{i}"))));
        let o = Lex(LocalTerm::triple(body(i, depth)));
        g.insert(Triple(s, p, o));
    }
    g
}

fn bench_triple_term_iso(c: &mut Criterion) {
    let mut group = c.benchmark_group("triple_term_isomorphism");
    for &(n, depth) in &[(1000usize, 1usize), (100, 2), (20, 3)] {
        let a = make_graph(n, depth);
        let b = make_graph(n, depth);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new(format!("depth{depth}"), n), &n, |bencher, _| {
            bencher.iter(|| {
                let r = graph_equivalent_with(&LexInterp, &a, &b);
                std::hint::black_box(r);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_triple_term_iso);
criterion_main!(benches);
