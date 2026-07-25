//! Literals are forbidden in subject position by RDF 1.1 §3.
use iri_rs::IriBuf;
use rdfx::{Literal, LocalTerm, Triple};

fn main() {
    let _t: Triple<Literal, IriBuf, LocalTerm> = Triple::new(
        Literal::plain("subj"),
        IriBuf::new("http://example.org/p".to_owned()).unwrap(),
        LocalTerm::iri(IriBuf::new("http://example.org/o".to_owned()).unwrap()),
    );
}
