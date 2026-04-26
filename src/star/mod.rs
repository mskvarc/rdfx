//! RDF 1.2 reification helpers — round-trippable mapping between
//! triple-term-bearing graphs/datasets and their RDF 1.1-compatible reified
//! form using the `rdf:reifies` annotation vocabulary.
//!
//! - [`unstar_graph`] / [`unstar_dataset`] unfold every triple-term occurrence
//!   into a reifier blank node with `rdf:type rdf:TripleTerm`,
//!   `rdf:subject`, `rdf:predicate`, `rdf:object` triples.
//! - [`restar_graph`] / [`restar_dataset`] detect such reified subgraphs and
//!   refold them back into [`LocalTerm::Triple`](crate::LocalTerm::Triple)
//!   objects.
//!
//! Round-trip: `unstar(restar(g)) ≡ g` and `restar(unstar(g)) ≡ g` modulo
//! blank renaming, verifiable via
//! [`graph_equivalent`](crate::dataset::isomorphism::graph_equivalent_with).

mod restar;
mod unstar;

pub use restar::{restar_dataset, restar_graph};
pub use unstar::{UnstarOptions, unstar_dataset, unstar_graph};

// Re-exports for convenience — the IRI constants live in `schema`.
pub use crate::{RDF_OBJECT, RDF_PREDICATE, RDF_REIFIES, RDF_SUBJECT, RDF_TRIPLE_TERM, RDF_TYPE};
