//! Literals are forbidden in graph-name position by RDF 1.1 §4.
use iri_rs::IriBuf;
use rdfx::{Id, Literal, LocalTerm, Quad};

fn main() {
    let _q: Quad<Id, IriBuf, LocalTerm, Literal> = Quad::new(
        Id::iri(IriBuf::new("http://example.org/s".to_owned()).unwrap()),
        IriBuf::new("http://example.org/p".to_owned()).unwrap(),
        LocalTerm::iri(IriBuf::new("http://example.org/o".to_owned()).unwrap()),
        Some(Literal::plain("graph")),
    );
}
