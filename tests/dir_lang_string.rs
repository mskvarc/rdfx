//! RDF 1.2 directional language-tagged string tests: construction,
//! equality, lexical form, and (with `serde`) round-trip.
#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use rdf_rs::{Direction, InvalidDirection, Literal, LiteralType, RdfDisplay, langtag::LangTagBuf};

fn en() -> LangTagBuf {
    "en".parse().unwrap()
}

#[test]
fn direction_parse_round_trip() {
    assert_eq!(Direction::parse("ltr").unwrap(), Direction::Ltr);
    assert_eq!(Direction::parse("rtl").unwrap(), Direction::Rtl);
    assert!(Direction::parse("nope").is_err());
    assert_eq!(Direction::Ltr.as_str(), "ltr");
    assert_eq!(Direction::Rtl.as_str(), "rtl");
}

#[test]
fn direction_parse_const_works() {
    assert_eq!(Direction::parse_const("ltr"), Some(Direction::Ltr));
    assert_eq!(Direction::parse_const("rtl"), Some(Direction::Rtl));
    assert_eq!(Direction::parse_const("xyz"), None);
}

#[test]
fn direction_invalid_carries_input() {
    let err: InvalidDirection = Direction::parse("xyz").unwrap_err();
    assert_eq!(err.0, "xyz");
}

#[test]
fn dir_lang_string_construction() {
    let lit = Literal::dir_lang_string("hi", en(), Direction::Ltr);
    assert_eq!(lit.as_str(), "hi");
    assert!(lit.is_dir_lang_string());
    assert!(lit.is_lang_string());
    assert_eq!(lit.lang_tag().unwrap().as_str(), "en");
    assert_eq!(lit.direction(), Some(Direction::Ltr));
}

#[test]
fn dir_lang_string_distinct_from_lang_string_at_same_value_and_tag() {
    let dir = Literal::dir_lang_string("hi", en(), Direction::Ltr);
    let plain_lang = Literal::lang_string("hi", en());
    assert_ne!(dir, plain_lang);
}

#[test]
fn dir_lang_string_distinct_by_direction() {
    let ltr = Literal::dir_lang_string("hi", en(), Direction::Ltr);
    let rtl = Literal::dir_lang_string("hi", en(), Direction::Rtl);
    assert_ne!(ltr, rtl);
}

#[test]
fn dir_lang_string_lexical_form_ltr() {
    let lit = Literal::dir_lang_string("hi", en(), Direction::Ltr);
    let s = format!("{}", lit.rdf_display());
    assert_eq!(s, r#""hi"@en--ltr"#);
}

#[test]
fn dir_lang_string_lexical_form_rtl() {
    let tag: LangTagBuf = "ar".parse().unwrap();
    let lit = Literal::dir_lang_string("مرحبا", tag, Direction::Rtl);
    let s = format!("{}", lit.rdf_display());
    assert_eq!(s, r#""مرحبا"@ar--rtl"#);
}

#[test]
fn lang_string_lexical_form_unchanged() {
    // Regression: existing lang-string lexical form must not change.
    let lit = Literal::lang_string("hi", en());
    assert_eq!(format!("{}", lit.rdf_display()), r#""hi"@en"#);
}

#[test]
fn literal_type_helpers_flag_dir_lang_string() {
    let lt = LiteralType::dir_lang_string(en(), Direction::Ltr);
    assert!(lt.is_lang_string());
    assert!(lt.is_dir_lang_string());
    assert!(!lt.is_undirected_lang_string());
    assert!(!lt.is_xsd_string());
    assert_eq!(lt.direction(), Some(Direction::Ltr));
    assert_eq!(lt.lang_tag().unwrap().as_str(), "en");
}

#[test]
fn dir_lang_string_via_macro() {
    use rdf_rs::triple;
    let t = triple![ <"http://ex/s"> <"http://ex/p"> "hello" @ "en" / "ltr" ];
    let lit = t.into_object().as_literal().unwrap().to_owned();
    assert!(lit.is_dir_lang_string());
    assert_eq!(lit.direction(), Some(Direction::Ltr));
}

#[cfg(feature = "serde")]
#[test]
fn direction_serde_round_trip() {
    let json = serde_json::to_string(&Direction::Ltr).unwrap();
    assert_eq!(json, r#""ltr""#);
    let parsed: Direction = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, Direction::Ltr);

    let parsed: Direction = serde_json::from_str(r#""rtl""#).unwrap();
    assert_eq!(parsed, Direction::Rtl);

    let bad: Result<Direction, _> = serde_json::from_str(r#""sideways""#);
    assert!(bad.is_err());
}

#[cfg(feature = "serde")]
#[test]
fn literal_type_serde_round_trip_dir_lang_string() {
    let lt = LiteralType::dir_lang_string(en(), Direction::Rtl);
    let json = serde_json::to_string(&lt).unwrap();
    let parsed: LiteralType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, lt);
}

#[cfg(feature = "serde")]
#[test]
fn literal_serde_round_trip_dir_lang_string() {
    let lit = Literal::dir_lang_string("hi", en(), Direction::Ltr);
    let json = serde_json::to_string(&lit).unwrap();
    let parsed: Literal = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, lit);
}
