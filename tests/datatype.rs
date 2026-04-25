#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]

use iri_rs::IriBuf;
use rdf_rs::{Datatype, Literal, LiteralType, RDF_LANG_STRING, RdfLangStringDatatypeError, XSD_STRING};

#[test]
fn datatype_new_rejects_rdf_lang_string() {
    let iri = IriBuf::from(RDF_LANG_STRING);
    let err = Datatype::new(iri).expect_err("rdf:langString must be rejected");
    let RdfLangStringDatatypeError(returned) = err;
    assert_eq!(returned.as_str(), RDF_LANG_STRING.as_str());
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
        LiteralType::LangString(_) => panic!("expected typed datatype"),
    }
}

#[test]
fn literal_type_try_from_iribuf() {
    let xsd: LiteralType = IriBuf::from(XSD_STRING).try_into().unwrap();
    assert!(xsd.is_xsd_string());
    let bad = LiteralType::try_from(IriBuf::from(RDF_LANG_STRING));
    assert!(bad.is_err());
}
