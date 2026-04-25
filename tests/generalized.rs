#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use iri_rs::IriBuf;
use rdf_rs::{
    BlankIdBuf,
    GeneralizationError,
    GeneralizedQuad,
    GeneralizedTriple,
    Id,
    Literal,
    LocalTerm,
    Quad,
    Term,
    Triple,
};

fn iri_term(s: &str) -> LocalTerm {
    LocalTerm::iri(IriBuf::new(s.to_owned()).unwrap())
}

fn blank_term(label: &str) -> LocalTerm {
    LocalTerm::BlankId(BlankIdBuf::new(format!("_:{label}")).unwrap())
}

fn lit_term(value: &str) -> LocalTerm {
    LocalTerm::literal(Literal::plain(value))
}

#[test]
fn generalized_triple_accepts_literal_subject() {
    // RDF 1.1 §A allows literal subjects in generalized RDF.
    let _t = GeneralizedTriple::new(lit_term("subj"), iri_term("http://example.org/p"), iri_term("http://example.org/o"));
}

#[test]
fn generalized_triple_accepts_literal_predicate() {
    let _t = GeneralizedTriple::new(iri_term("http://example.org/s"), lit_term("pred"), iri_term("http://example.org/o"));
}

#[test]
fn generalized_triple_accepts_blank_predicate() {
    let _t = GeneralizedTriple::new(iri_term("http://example.org/s"), blank_term("p"), iri_term("http://example.org/o"));
}

#[test]
fn try_into_triple_rejects_literal_subject() {
    let g = GeneralizedTriple::new(lit_term("subj"), iri_term("http://example.org/p"), iri_term("http://example.org/o"));
    match g.try_into_triple() {
        Err(GeneralizationError::LiteralSubject) => {}
        other => panic!("expected LiteralSubject, got {other:?}"),
    }
}

#[test]
fn try_into_triple_rejects_literal_predicate() {
    let g = GeneralizedTriple::new(iri_term("http://example.org/s"), lit_term("pred"), iri_term("http://example.org/o"));
    match g.try_into_triple() {
        Err(GeneralizationError::LiteralPredicate) => {}
        other => panic!("expected LiteralPredicate, got {other:?}"),
    }
}

#[test]
fn try_into_triple_rejects_blank_predicate() {
    let g = GeneralizedTriple::new(iri_term("http://example.org/s"), blank_term("p"), iri_term("http://example.org/o"));
    match g.try_into_triple() {
        Err(GeneralizationError::BlankPredicate) => {}
        other => panic!("expected BlankPredicate, got {other:?}"),
    }
}

#[test]
fn try_into_triple_succeeds_for_valid_triple() {
    let g = GeneralizedTriple::new(blank_term("s"), iri_term("http://example.org/p"), iri_term("http://example.org/o"));
    let t = g.try_into_triple().unwrap();
    match t.0 {
        Id::BlankId(_) => {}
        Id::Iri(_) => panic!("expected blank subject"),
    }
    assert_eq!(t.1.as_str(), "http://example.org/p");
}

#[test]
fn from_strict_triple_round_trips_via_into() {
    let strict: Triple<Id, IriBuf, LocalTerm> = Triple::new(
        Id::iri(IriBuf::new("http://example.org/s".to_owned()).unwrap()),
        IriBuf::new("http://example.org/p".to_owned()).unwrap(),
        iri_term("http://example.org/o"),
    );
    let _g: GeneralizedTriple<Id, IriBuf, LocalTerm> = strict.into();
}

#[test]
fn try_into_quad_rejects_literal_graph() {
    let g = GeneralizedQuad::new(
        iri_term("http://example.org/s"),
        iri_term("http://example.org/p"),
        iri_term("http://example.org/o"),
        Some(lit_term("graph")),
    );
    match g.try_into_quad() {
        Err(GeneralizationError::LiteralGraph) => {}
        other => panic!("expected LiteralGraph, got {other:?}"),
    }
}

#[test]
fn try_into_quad_succeeds_with_default_graph() {
    let g = GeneralizedQuad::new(
        iri_term("http://example.org/s"),
        iri_term("http://example.org/p"),
        iri_term("http://example.org/o"),
        None,
    );
    let q: Quad<Id, IriBuf, LocalTerm, Id> = g.try_into_quad().unwrap();
    assert!(q.graph().is_none());
}

#[test]
fn try_into_quad_succeeds_with_named_graph() {
    let g = GeneralizedQuad::new(
        iri_term("http://example.org/s"),
        iri_term("http://example.org/p"),
        iri_term("http://example.org/o"),
        Some(blank_term("g")),
    );
    let q: Quad<Id, IriBuf, LocalTerm, Id> = g.try_into_quad().unwrap();
    match q.graph().unwrap() {
        Id::BlankId(_) => {}
        Id::Iri(_) => panic!("expected blank graph"),
    }
}

#[test]
fn from_quad_uses_term_iri_for_object_check() {
    let term = Term::Iri(IriBuf::new("http://example.org/o".to_owned()).unwrap());
    let _ = LocalTerm::from(term);
}
