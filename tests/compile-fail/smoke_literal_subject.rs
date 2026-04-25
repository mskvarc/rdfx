//! Plan K smoke test: the canonical example
//!   `let t = Triple::new(Literal::plain("x"), IriBuf::from(iri!("...")), Literal::plain("y"));`
//! must fail to compile with `Literal: IsSubject` not satisfied.
use rdf_rs::{Literal, Triple, iri};
use iri_rs::IriBuf;

fn main() {
    let _t = Triple::new(
        Literal::plain("x"),
        IriBuf::from(iri!("http://ex/p")),
        Literal::plain("y"),
    );
}
