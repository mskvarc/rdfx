#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use std::str::FromStr;

use rdf_rs::{Literal, LiteralType, LocalTerm, Term, generalized_quad, generalized_triple, langtag::LangTagBuf, quad, triple};

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
