#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use iri_rs::IriBuf;
use rdf_rs::{Datatype, Direction, Literal, LiteralType, RDF_DIR_LANG_STRING, RDF_LANG_STRING, ReservedDatatypeError, XSD_STRING, langtag::LangTagBuf};

#[test]
fn datatype_new_rejects_rdf_lang_string() {
    let iri = IriBuf::from(RDF_LANG_STRING);
    let err = Datatype::new(iri).expect_err("rdf:langString must be rejected");
    assert!(matches!(err, ReservedDatatypeError::LangString(_)));
    assert_eq!(err.iri().as_str(), RDF_LANG_STRING.as_str());
}

#[test]
fn datatype_new_rejects_rdf_dir_lang_string() {
    let iri = IriBuf::from(RDF_DIR_LANG_STRING);
    let err = Datatype::new(iri).expect_err("rdf:dirLangString must be rejected");
    assert!(matches!(err, ReservedDatatypeError::DirLangString(_)));
    assert_eq!(err.iri().as_str(), RDF_DIR_LANG_STRING.as_str());
}

#[test]
fn datatype_new_accepts_xsd_string() {
    let iri = IriBuf::from(XSD_STRING);
    let dt = Datatype::new(iri).unwrap();
    assert_eq!(dt.as_iri().as_str(), XSD_STRING.as_str());
}

#[test]
fn datatype_xsd_string_round_trip() {
    let dt = Datatype::xsd_string();
    assert_eq!(dt.as_iri().as_str(), XSD_STRING.as_str());
    let iri = dt.into_iri();
    assert_eq!(iri.as_str(), XSD_STRING.as_str());
}

#[test]
fn literal_type_typed_rejects_rdf_lang_string() {
    let iri = IriBuf::from(RDF_LANG_STRING);
    assert!(LiteralType::typed(iri).is_err());
}

#[test]
fn literal_type_typed_rejects_rdf_dir_lang_string() {
    let iri = IriBuf::from(RDF_DIR_LANG_STRING);
    assert!(LiteralType::typed(iri).is_err());
}

#[test]
fn literal_type_xsd_string_constructs_any_variant() {
    let lt = LiteralType::xsd_string();
    assert!(matches!(lt, LiteralType::Any(_)));
    assert!(lt.is_xsd_string());
}

#[test]
fn literal_plain_equals_literal_typed_xsd_string() {
    let plain = Literal::plain("hello");
    let typed = Literal::typed("hello", Datatype::xsd_string());
    assert_eq!(plain, typed);
}

#[test]
fn literal_plain_uses_xsd_string_datatype() {
    let lit = Literal::plain("x");
    assert!(lit.as_type().is_xsd_string());
    assert_eq!(lit.as_str(), "x");
}

#[test]
fn literal_typed_with_custom_datatype_round_trips() {
    let dt_iri = IriBuf::new("http://example.org/dt".to_owned()).unwrap();
    let dt = Datatype::new(dt_iri.clone()).unwrap();
    let lit = Literal::typed("42", dt);
    match lit.as_type() {
        LiteralType::Any(d) => assert_eq!(d.as_iri().as_str(), dt_iri.as_str()),
        _ => panic!("expected typed datatype"),
    }
}

#[test]
fn literal_type_try_from_iribuf() {
    let xsd: LiteralType = IriBuf::from(XSD_STRING).try_into().unwrap();
    assert!(xsd.is_xsd_string());
    let bad = LiteralType::try_from(IriBuf::from(RDF_LANG_STRING));
    assert!(bad.is_err());
    let bad_dir = LiteralType::try_from(IriBuf::from(RDF_DIR_LANG_STRING));
    assert!(bad_dir.is_err());
}

#[test]
fn dir_lang_string_distinct_from_lang_string() {
    let tag: LangTagBuf = "en".parse().unwrap();
    let lang = Literal::lang_string("hi", tag.clone());
    let dir = Literal::dir_lang_string("hi", tag, Direction::Ltr);
    assert_ne!(lang, dir, "DirLangString must not equal LangString even at same value/tag");
    assert!(lang.is_lang_string());
    assert!(dir.is_lang_string());
    assert!(lang.is_undirected_lang_string());
    assert!(!dir.is_undirected_lang_string());
    assert!(!lang.is_dir_lang_string());
    assert!(dir.is_dir_lang_string());
    assert_eq!(lang.direction(), None);
    assert_eq!(dir.direction(), Some(Direction::Ltr));
}

#[cfg(feature = "serde")]
#[test]
fn datatype_serde_rejects_rdf_lang_string() {
    let json = r#""http://www.w3.org/1999/02/22-rdf-syntax-ns#langString""#;
    let result: Result<Datatype, _> = serde_json::from_str(json);
    assert!(result.is_err(), "deserializing rdf:langString into Datatype must fail");
}

#[cfg(feature = "serde")]
#[test]
fn datatype_serde_rejects_rdf_dir_lang_string() {
    let json = r#""http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString""#;
    let result: Result<Datatype, _> = serde_json::from_str(json);
    assert!(result.is_err(), "deserializing rdf:dirLangString into Datatype must fail");
}

#[cfg(feature = "serde")]
#[test]
fn datatype_serde_round_trip_xsd_string() {
    let dt = Datatype::xsd_string();
    let json = serde_json::to_string(&dt).unwrap();
    let parsed: Datatype = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, dt);
}

#[cfg(feature = "serde")]
#[test]
fn literal_type_serde_rejects_rdf_lang_string_in_any() {
    let json = r#"{"Any":"http://www.w3.org/1999/02/22-rdf-syntax-ns#langString"}"#;
    let result: Result<LiteralType, _> = serde_json::from_str(json);
    assert!(result.is_err(), "deserializing Any(rdf:langString) must fail");
}

#[cfg(feature = "serde")]
#[test]
fn literal_type_serde_rejects_rdf_dir_lang_string_in_any() {
    let json = r#"{"Any":"http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString"}"#;
    let result: Result<LiteralType, _> = serde_json::from_str(json);
    assert!(result.is_err(), "deserializing Any(rdf:dirLangString) must fail");
}
