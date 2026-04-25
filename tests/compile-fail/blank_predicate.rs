//! Blank nodes are forbidden in predicate position by RDF 1.1 §3.
use rdf_rs::{BlankIdBuf, Id, IriBuf, LocalTerm, Triple};

fn main() {
    let _t: Triple<Id, BlankIdBuf, LocalTerm> = Triple::new(
        Id::iri(IriBuf::new("http://example.org/s".to_owned()).unwrap()),
        BlankIdBuf::new("_:b0".to_owned()).unwrap(),
        LocalTerm::iri(IriBuf::new("http://example.org/o".to_owned()).unwrap()),
    );
}
