#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use std::str::FromStr;

use rdfx::{Direction, Literal, LiteralType, LocalTerm, Term, generalized_quad, generalized_triple, langtag::LangTagBuf, quad, triple, triples};

#[test]
fn triple_macro_lang_string_object() {
    let t = triple!(<"http://ex/s"> <"http://ex/p"> "bonjour" @ "fr");
    match t.into_object() {
        LocalTerm::Named(Term::Literal(lit)) => {
            assert_eq!(lit.as_str(), "bonjour");
            assert_eq!(lit.lang_tag().unwrap().as_str(), "fr");
            assert!(matches!(lit.into_type(), LiteralType::LangString(_)));
        }
        other => panic!("expected lang-string literal, got {other:?}"),
    }
}

#[test]
fn quad_macro_lang_string_object() {
    let q = quad!(<"http://ex/s"> <"http://ex/p"> "guten tag" @ "de-AT" <"http://ex/g">);
    match q.into_object() {
        LocalTerm::Named(Term::Literal(lit)) => {
            assert_eq!(lit.as_str(), "guten tag");
            assert_eq!(lit.lang_tag().unwrap().as_str(), "de-AT");
        }
        other => panic!("expected lang-string literal, got {other:?}"),
    }
}

#[test]
fn generalized_triple_lang_string() {
    let t = generalized_triple!("hi" @ "en" <"http://ex/p"> "world" @ "en-US");
    let s = t.subject();
    let o = t.object();
    let LocalTerm::Named(Term::Literal(s_lit)) = s else {
        panic!("subject not literal")
    };
    let LocalTerm::Named(Term::Literal(o_lit)) = o else {
        panic!("object not literal")
    };
    assert_eq!(s_lit.lang_tag().unwrap().as_str(), "en");
    assert_eq!(o_lit.lang_tag().unwrap().as_str(), "en-US");
}

#[test]
fn generalized_quad_lang_string() {
    let q = generalized_quad!(<"http://ex/s"> <"http://ex/p"> "bonjour" @ "fr-CA" <"http://ex/g">);
    let LocalTerm::Named(Term::Literal(lit)) = q.into_object() else {
        panic!("object not literal")
    };
    assert_eq!(lit.lang_tag().unwrap().as_str(), "fr-CA");
}

#[test]
fn macro_lang_string_matches_constructor() {
    let from_macro = match triple!(<"http://ex/s"> <"http://ex/p"> "x" @ "en").into_object() {
        LocalTerm::Named(Term::Literal(l)) => l,
        _ => panic!(),
    };
    let from_ctor = Literal::lang_string("x", LangTagBuf::from_str("en").unwrap());
    assert_eq!(from_macro, from_ctor);
}

// ----- RDF 1.2 triple-term and dir-lang-string macro tests -----

#[test]
fn triple_macro_triple_term_object() {
    let t = triple!(
        <"http://ex/a"> <"http://ex/says">
            <<( <"http://ex/s"> <"http://ex/p"> <"http://ex/o"> )>>
    );
    let body = t.into_object().into_triple().expect("triple-term object");
    assert_eq!(body.subject().as_iri().unwrap().as_str(), "http://ex/s");
    assert_eq!(body.predicate().as_str(), "http://ex/p");
    assert_eq!(body.object().as_iri().unwrap().as_str(), "http://ex/o");
}

#[test]
fn triple_macro_nested_triple_term_object() {
    let t = triple!(
        <"http://ex/a"> <"http://ex/says">
            <<(
                <"http://ex/b"> <"http://ex/says">
                    <<( <"http://ex/s"> <"http://ex/p"> <"http://ex/o"> )>>
            )>>
    );
    let outer = t.into_object().into_triple().unwrap();
    let inner = outer.into_object().into_triple().unwrap();
    assert_eq!(inner.subject().as_iri().unwrap().as_str(), "http://ex/s");
}

#[test]
fn quad_macro_triple_term_object_with_graph() {
    let q = quad!(
        <"http://ex/a"> <"http://ex/says">
            <<( <"http://ex/s"> <"http://ex/p"> <"http://ex/o"> )>>
        <"http://ex/g">
    );
    assert!(q.object().is_triple());
    assert_eq!(q.graph().unwrap().as_iri().unwrap().as_str(), "http://ex/g");
}

#[test]
fn triple_macro_dir_lang_string_object_ltr() {
    let t = triple!(<"http://ex/s"> <"http://ex/p"> "hello" @ "en" / "ltr");
    let lit = t.into_object().as_literal().unwrap().to_owned();
    assert!(lit.is_dir_lang_string());
    assert_eq!(lit.lang_tag().unwrap().as_str(), "en");
    assert_eq!(lit.direction(), Some(Direction::Ltr));
}

#[test]
fn triple_macro_dir_lang_string_object_rtl() {
    let t = triple!(<"http://ex/s"> <"http://ex/p"> "مرحبا" @ "ar" / "rtl");
    let lit = t.into_object().as_literal().unwrap().to_owned();
    assert_eq!(lit.direction(), Some(Direction::Rtl));
    match lit.into_type() {
        LiteralType::DirLangString { tag, direction } => {
            assert_eq!(tag.as_str(), "ar");
            assert_eq!(direction, Direction::Rtl);
        }
        _ => panic!("expected DirLangString"),
    }
}

#[test]
fn quad_macro_dir_lang_string_object_with_graph() {
    let q = quad!(<"http://ex/s"> <"http://ex/p"> "hello" @ "en" / "ltr" <"http://ex/g">);
    let lit = q.into_object().as_literal().unwrap().to_owned();
    assert!(lit.is_dir_lang_string());
}

#[test]
fn triples_array_with_triple_term_and_dir_lang() {
    let arr = triples![
        <"http://ex/a"> <"http://ex/says">
            <<( <"http://ex/s"> <"http://ex/p"> <"http://ex/o"> )>> .
        <"http://ex/a"> <"http://ex/has-greeting"> "hello" @ "en" / "ltr" .
        <"http://ex/b"> <"http://ex/p"> "plain" .
    ];
    assert_eq!(arr.len(), 3);
    assert!(arr[0].object().is_triple());
    assert!(arr[1].object().as_literal().unwrap().is_dir_lang_string());
}

#[test]
fn generalized_triple_with_triple_term_in_subject_position() {
    // Generalized macros allow triple terms anywhere — the body is built via
    // strict `triple!` so the inner body must still be strict.
    let t = generalized_triple!(
        <<( <"http://ex/s"> <"http://ex/p"> <"http://ex/o"> )>>
        <"http://ex/asserted-by">
        <"http://ex/alice">
    );
    assert!(t.subject().is_triple());
}
