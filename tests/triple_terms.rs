//! RDF 1.2 triple-term type-model tests: construction, equality, hashing,
//! lexical form (RdfDisplay), conversions, generalized variants.
#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use iri_rs::{IriBuf, iri};
use rdf_rs::{
    BlankIdBuf, FullyGeneralizedTriple, GeneralizationError, GeneralizedLocalTerm, GeneralizedTriple, Id, Literal, LocalTerm, LocalTermRef, RdfDisplay, Term, Triple, TripleTerm, triple,
};

fn ex(s: &str) -> IriBuf {
    IriBuf::new(format!("http://example.org/{s}")).unwrap()
}

fn s_iri(s: &str) -> Id {
    Id::iri(ex(s))
}

fn o_iri(s: &str) -> LocalTerm {
    LocalTerm::iri(ex(s))
}

#[test]
fn construct_triple_term_via_macro() {
    let t = triple![ <"http://ex/a"> <"http://ex/says"> <<( <"http://ex/s"> <"http://ex/p"> <"http://ex/o"> )>> ];
    assert!(t.object().is_triple());
    let body = t.object().as_triple().unwrap();
    assert_eq!(body.predicate().as_str(), "http://ex/p");
}

#[test]
fn construct_triple_term_via_constructor() {
    let body: TripleTerm = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let term = LocalTerm::triple(body.clone());
    assert!(term.is_triple());
    assert_eq!(term.as_triple(), Some(&body));
    assert_eq!(term.into_triple(), Some(body));
}

#[test]
fn triple_term_equality_and_hashing() {
    let a = LocalTerm::triple(Triple(s_iri("s"), ex("p"), o_iri("o")));
    let b = LocalTerm::triple(Triple(s_iri("s"), ex("p"), o_iri("o")));
    assert_eq!(a, b);
    let h = |t: &LocalTerm| {
        let mut h = DefaultHasher::new();
        t.hash(&mut h);
        h.finish()
    };
    assert_eq!(h(&a), h(&b));
}

#[test]
fn triple_term_inequality_by_component() {
    let a = LocalTerm::triple(Triple(s_iri("s"), ex("p"), o_iri("o")));
    let b = LocalTerm::triple(Triple(s_iri("s2"), ex("p"), o_iri("o")));
    assert_ne!(a, b);
}

#[test]
fn triple_term_hash_distinct_from_blank_with_same_repr() {
    let blank = LocalTerm::BlankId(BlankIdBuf::new("_:tt".to_owned()).unwrap());
    let term = LocalTerm::triple(Triple(s_iri("s"), ex("p"), o_iri("o")));
    let h = |t: &LocalTerm| {
        let mut h = DefaultHasher::new();
        t.hash(&mut h);
        h.finish()
    };
    assert_ne!(h(&blank), h(&term), "discriminant byte must distinguish variants");
}

#[test]
fn triple_term_rdf_display_uses_double_angle_parens() {
    let body = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let term = LocalTerm::triple(body);
    let s = format!("{}", term.rdf_display());
    assert_eq!(s, "<<( <http://example.org/s> <http://example.org/p> <http://example.org/o> )>>");
}

#[test]
fn nested_triple_term_rdf_display() {
    let inner = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let outer = Triple(s_iri("a"), ex("says"), LocalTerm::triple(inner));
    let term = LocalTerm::triple(outer);
    let s = format!("{}", term.rdf_display());
    assert!(s.contains("<<( <http://example.org/a>"));
    assert!(s.contains("<<( <http://example.org/s>"));
    assert!(s.ends_with(")>> )>>"));
}

#[test]
fn triple_term_as_ref_and_to_owned_round_trip() {
    let term = LocalTerm::triple(Triple(s_iri("s"), ex("p"), o_iri("o")));
    let r: LocalTermRef<'_> = term.as_ref();
    assert!(r.is_triple());
    assert_eq!(r.to_owned(), term);
}

#[test]
fn try_into_triple_negative() {
    let blank = LocalTerm::BlankId(BlankIdBuf::new("_:b0".to_owned()).unwrap());
    let result = blank.clone().try_into_triple();
    assert!(result.is_err());
    let original = result.unwrap_err();
    assert_eq!(original, blank);
}

#[test]
fn from_triple_term_into_local_term() {
    let body = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let lt: LocalTerm = body.clone().into();
    assert_eq!(lt, LocalTerm::triple(body));
}

// ----- Generalized: literal subject inside triple-term body via FullyGeneralized -----

#[test]
fn fully_generalized_triple_with_literal_subject_inside_body() {
    // Outer: <"a"> <"p"> <<( "lit-subj" <"q"> "v" )>>
    let inner_subject = GeneralizedLocalTerm::literal(Literal::plain("lit-subj"));
    let inner_predicate = GeneralizedLocalTerm::iri(ex("q"));
    let inner_object = GeneralizedLocalTerm::literal(Literal::plain("v"));
    let inner = GeneralizedTriple::new(inner_subject, inner_predicate, inner_object);

    let s = GeneralizedLocalTerm::iri(ex("a"));
    let p = GeneralizedLocalTerm::iri(ex("p"));
    let o = GeneralizedLocalTerm::triple(inner);
    let outer: FullyGeneralizedTriple = GeneralizedTriple::new(s, p, o);

    // Narrowing must fail because the inner subject is a literal.
    let result = outer.try_into_strict_triple();
    assert!(matches!(result, Err(GeneralizationError::LiteralSubject)));
}

#[test]
fn fully_generalized_triple_strict_triple_term_narrows() {
    // Outer: <"a"> <"says"> <<( <"s"> <"p"> <"o"> )>> — inner body strict, narrows.
    let inner = GeneralizedTriple::new(
        GeneralizedLocalTerm::iri(ex("s")),
        GeneralizedLocalTerm::iri(ex("p")),
        GeneralizedLocalTerm::iri(ex("o")),
    );
    let outer: FullyGeneralizedTriple = GeneralizedTriple::new(
        GeneralizedLocalTerm::iri(ex("a")),
        GeneralizedLocalTerm::iri(ex("says")),
        GeneralizedLocalTerm::triple(inner),
    );
    let strict = outer.try_into_strict_triple().unwrap();
    assert!(strict.object().is_triple());
}

#[test]
fn from_local_term_into_generalized_local_term_preserves_triple() {
    let body = Triple(s_iri("s"), ex("p"), o_iri("o"));
    let lt = LocalTerm::triple(body.clone());
    let g: GeneralizedLocalTerm = lt.into();
    assert!(g.is_triple());
}

#[test]
fn rdf_iri_const_used_for_construction() {
    // Sanity: smoke the iri! macro path through TripleTerm.
    let p = IriBuf::from(iri!("http://example.org/has"));
    let t = LocalTerm::triple(Triple(s_iri("s"), p, o_iri("o")));
    let inner = t.as_triple().unwrap();
    assert_eq!(inner.predicate().as_str(), "http://example.org/has");
}

#[test]
fn triple_term_in_object_via_term_named_does_not_misclassify() {
    let named = LocalTerm::Named(Term::Iri(ex("plain")));
    assert!(!named.is_triple());
    assert_eq!(named.as_triple(), None);
}
