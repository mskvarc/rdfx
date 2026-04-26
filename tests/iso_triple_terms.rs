//! RDF 1.2 isomorphism tests targeting `Bijection::resource_matches_with`'s
//! triple-term descent and the `ReverseInterpretation::triple_term_components`
//! override.
//!
//! Uses a `Lex(LocalTerm)` newtype declared `Resource` via `impl_resource!`
//! so that the uniform-graph storage [`BTreeGraph<Lex>`] can hold lexical
//! triple-term resources, and a custom `LexInterp` interpretation that
//! reports IRIs / literals / triple-term components faithfully.
#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use std::borrow::Cow;

use iri_rs::IriBuf;
use rdf_rs::{
    BlankIdBuf,
    CowLiteral,
    Id,
    Interpretation,
    LiteralRef,
    LocalTerm,
    Term,
    Triple,
    TripleTerm,
    dataset::{
        BTreeGraph,
        isomorphism::{graph_equivalent, graph_equivalent_with},
    },
    impl_resource,
    interpretation::ReverseInterpretation,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Lex(LocalTerm);

impl_resource!(Lex);

impl rdf_rs::RdfDisplay for Lex {
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
}

fn ex(s: &str) -> IriBuf {
    IriBuf::new(format!("http://example.org/{s}")).unwrap()
}

fn iri_lex(s: &str) -> Lex {
    Lex(LocalTerm::iri(ex(s)))
}

fn blank_lex(s: &str) -> Lex {
    Lex(LocalTerm::BlankId(BlankIdBuf::new(format!("_:{s}")).unwrap()))
}

fn tt_lex(body: TripleTerm) -> Lex {
    Lex(LocalTerm::triple(body))
}

fn graph_with(triples: &[(Lex, Lex, Lex)]) -> BTreeGraph<Lex> {
    let mut g: BTreeGraph<Lex> = BTreeGraph::new();
    for (s, p, o) in triples {
        g.insert(Triple(s.clone(), p.clone(), o.clone()));
    }
    g
}

// ----- tests -----

#[test]
fn triple_terms_with_identical_ground_components_are_equivalent() {
    let body: TripleTerm = Triple(Id::iri(ex("s")), ex("p"), LocalTerm::iri(ex("o")));
    let a = graph_with(&[(iri_lex("a"), iri_lex("says"), tt_lex(body.clone()))]);
    let b = graph_with(&[(iri_lex("a"), iri_lex("says"), tt_lex(body))]);
    assert!(graph_equivalent_with(&LexInterp, &a, &b));
}

#[test]
fn triple_terms_differing_by_iri_are_not_equivalent() {
    let body_a: TripleTerm = Triple(Id::iri(ex("s")), ex("p"), LocalTerm::iri(ex("o")));
    let body_b: TripleTerm = Triple(Id::iri(ex("s")), ex("p"), LocalTerm::iri(ex("DIFFERENT")));
    let a = graph_with(&[(iri_lex("a"), iri_lex("says"), tt_lex(body_a))]);
    let b = graph_with(&[(iri_lex("a"), iri_lex("says"), tt_lex(body_b))]);
    assert!(!graph_equivalent_with(&LexInterp, &a, &b));
}

#[test]
fn triple_terms_blank_subject_aligned_via_outer_bijection() {
    // The blank inside the triple-term body is also referenced outside.
    // Bijection slot allocated for the outer occurrence; component recursion
    // verifies the inner occurrence matches via Borrow-based map lookup.
    let blank_a = Id::BlankId(BlankIdBuf::new("_:x".to_owned()).unwrap());
    let blank_b = Id::BlankId(BlankIdBuf::new("_:y".to_owned()).unwrap());
    let body_a: TripleTerm = Triple(blank_a, ex("p"), LocalTerm::iri(ex("o")));
    let body_b: TripleTerm = Triple(blank_b, ex("p"), LocalTerm::iri(ex("o")));

    let a = graph_with(&[
        (iri_lex("a"), iri_lex("says"), tt_lex(body_a)),
        (Lex(LocalTerm::BlankId(BlankIdBuf::new("_:x".to_owned()).unwrap())), iri_lex("p"), iri_lex("z")),
    ]);
    let b = graph_with(&[
        (iri_lex("a"), iri_lex("says"), tt_lex(body_b)),
        (Lex(LocalTerm::BlankId(BlankIdBuf::new("_:y".to_owned()).unwrap())), iri_lex("p"), iri_lex("z")),
    ]);
    assert!(graph_equivalent_with(&LexInterp, &a, &b));
}

#[test]
fn nested_triple_terms_match_componentwise() {
    let inner: TripleTerm = Triple(Id::iri(ex("s")), ex("p"), LocalTerm::iri(ex("o")));
    let outer: TripleTerm = Triple(Id::iri(ex("a")), ex("says"), LocalTerm::triple(inner));
    let g1 = graph_with(&[(iri_lex("z"), iri_lex("witnesses"), tt_lex(outer.clone()))]);
    let g2 = graph_with(&[(iri_lex("z"), iri_lex("witnesses"), tt_lex(outer))]);
    assert!(graph_equivalent_with(&LexInterp, &g1, &g2));
}

#[test]
fn nested_triple_terms_differ_at_inner_iri() {
    let inner_a: TripleTerm = Triple(Id::iri(ex("s")), ex("p"), LocalTerm::iri(ex("o")));
    let inner_b: TripleTerm = Triple(Id::iri(ex("s")), ex("p"), LocalTerm::iri(ex("X")));
    let outer_a: TripleTerm = Triple(Id::iri(ex("a")), ex("says"), LocalTerm::triple(inner_a));
    let outer_b: TripleTerm = Triple(Id::iri(ex("a")), ex("says"), LocalTerm::triple(inner_b));
    let g1 = graph_with(&[(iri_lex("z"), iri_lex("witnesses"), tt_lex(outer_a))]);
    let g2 = graph_with(&[(iri_lex("z"), iri_lex("witnesses"), tt_lex(outer_b))]);
    assert!(!graph_equivalent_with(&LexInterp, &g1, &g2));
}

#[test]
fn structural_iso_without_lex_interp_is_blind_to_triple_term_contents() {
    // Without `LexInterp`, the no-`_with` entry point uses BlindInterpretation —
    // it sees triple-term resources as opaque blank-likes and freely renames
    // them. Two graphs that differ only in triple-term *contents* are still
    // structurally equivalent under that lens.
    let body_a: TripleTerm = Triple(Id::iri(ex("s")), ex("p"), LocalTerm::iri(ex("o")));
    let body_b: TripleTerm = Triple(Id::iri(ex("s")), ex("p"), LocalTerm::iri(ex("o2")));
    let a = graph_with(&[(iri_lex("a"), iri_lex("says"), tt_lex(body_a))]);
    let b = graph_with(&[(iri_lex("a"), iri_lex("says"), tt_lex(body_b))]);
    assert!(graph_equivalent(&a, &b), "BlindInterpretation does not descend into triple-term contents");
}

#[test]
fn unused_blank_helper_compiles() {
    // smoke: blank_lex helper is reachable
    let _ = blank_lex("foo");
}

// ----- triple-term-only blanks -----

#[test]
fn blanks_only_inside_triple_terms_are_tracked() {
    // No outer blanks; the only blanks live inside the triple-term bodies.
    // Each side has one such blank; the bijection must allocate a slot for it.
    let body_a: TripleTerm = Triple(Id::BlankId(BlankIdBuf::new("_:x".to_owned()).unwrap()), iri_lex_iri("p"), iri_lex_lt("o"));
    let body_b: TripleTerm = Triple(Id::BlankId(BlankIdBuf::new("_:y".to_owned()).unwrap()), iri_lex_iri("p"), iri_lex_lt("o"));

    let a = graph_with(&[(iri_lex("a"), iri_lex("says"), tt_lex(body_a))]);
    let b = graph_with(&[(iri_lex("a"), iri_lex("says"), tt_lex(body_b))]);
    assert!(graph_equivalent_with(&LexInterp, &a, &b));
}

#[test]
fn triple_term_internal_blank_count_mismatch_rejected() {
    // a's inner triple-term has 2 distinct blanks; b's inner has 1 (reused).
    // Interior blank counting catches this before the bijection runs.
    let body_a: TripleTerm = Triple(
        Id::BlankId(BlankIdBuf::new("_:x".to_owned()).unwrap()),
        iri_lex_iri("p"),
        LocalTerm::BlankId(BlankIdBuf::new("_:y".to_owned()).unwrap()),
    );
    let body_b: TripleTerm = Triple(
        Id::BlankId(BlankIdBuf::new("_:z".to_owned()).unwrap()),
        iri_lex_iri("p"),
        LocalTerm::BlankId(BlankIdBuf::new("_:z".to_owned()).unwrap()),
    );

    let a = graph_with(&[(iri_lex("a"), iri_lex("says"), tt_lex(body_a))]);
    let b = graph_with(&[(iri_lex("a"), iri_lex("says"), tt_lex(body_b))]);

    // Note: blank_count in `a` = 2 (distinct _:x, _:y); in `b` it's also 2
    // because we count occurrences, not distinct ids. So count matches but
    // structural shape differs — bijection should reject.
    assert!(!graph_equivalent_with(&LexInterp, &a, &b));
}

#[test]
fn nested_triple_term_interior_blank_must_align() {
    // Interior blank in inner-most triple-term body. Needs descent past 2 levels.
    let inner_a: TripleTerm = Triple(
        Id::BlankId(BlankIdBuf::new("_:x".to_owned()).unwrap()),
        iri_lex_iri("p"),
        iri_lex_lt("o"),
    );
    let inner_b: TripleTerm = Triple(
        Id::BlankId(BlankIdBuf::new("_:y".to_owned()).unwrap()),
        iri_lex_iri("p"),
        iri_lex_lt("o"),
    );
    let outer_a: TripleTerm = Triple(Id::iri(ex("o1")), iri_lex_iri("witnesses"), LocalTerm::triple(inner_a));
    let outer_b: TripleTerm = Triple(Id::iri(ex("o1")), iri_lex_iri("witnesses"), LocalTerm::triple(inner_b));

    let a = graph_with(&[(iri_lex("z"), iri_lex("says"), tt_lex(outer_a))]);
    let b = graph_with(&[(iri_lex("z"), iri_lex("says"), tt_lex(outer_b))]);
    assert!(graph_equivalent_with(&LexInterp, &a, &b));
}

// helpers used above
fn iri_lex_iri(s: &str) -> IriBuf {
    ex(s)
}

fn iri_lex_lt(s: &str) -> LocalTerm {
    LocalTerm::iri(ex(s))
}
