//! Trybuild compile-fail harness for the RDF 1.1 position invariants.
//!
//! Each `tests/compile-fail/*.rs` file constructs an invalid `Triple`/`Quad`
//! at the type level (literal subject/predicate/graph, blank predicate). The
//! position-marker traits should refuse to compile each one.
//!
//! Gated on the `meta` feature because the `.stderr` snapshots include the
//! `Meta<T, M>` line in trait-implementor lists, which is only present when
//! the `meta` feature is enabled. Run with `--all-features` (or at least
//! `--features meta`) to exercise this test.

#[cfg(feature = "meta")]
#[test]
fn position_invariants_reject_invalid_terms() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
