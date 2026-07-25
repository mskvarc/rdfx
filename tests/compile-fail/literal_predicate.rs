//! Literals are forbidden in predicate position by RDF 1.1 §3.
use iri_rs::IriBuf;
use rdfx::{Id, Literal, LocalTerm, Triple};

fn main() {
    let _t: Triple<Id, Literal, LocalTerm> = Triple::new(
        Id::iri(IriBuf::new("http://example.org/s".to_owned()).unwrap()),
        Literal::plain("pred"),
        LocalTerm::iri(IriBuf::new("http://example.org/o".to_owned()).unwrap()),
    );
}
