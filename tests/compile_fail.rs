//! Trybuild compile-fail harness for the RDF 1.1 position invariants.
//!
//! Each `tests/compile-fail/*.rs` file constructs an invalid `Triple`/`Quad`
//! at the type level (literal subject/predicate/graph, blank predicate). The
//! position-marker traits should refuse to compile each one.

#[test]
fn position_invariants_reject_invalid_terms() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
