//! The [Resource Description Framework (RDF)][rdf] is a very simple graph data
//! model defined by the [World Wide Web Consortium (W3C)][w3c] to represent
//! arbitrary pieces of information, primarily intended for the web. Nodes of
//! the graph are called *resources*, and resources are connected together using
//! *relations*, which are resources themselves.
//!
//! This is a utility library providing common types, data-structures, traits,
//! constants and macro definitions to deal with RDF data:
//! - IRIs (through the [`iri-rs`](iri_rs) crate), blank node identifiers and
//!   literals to represent resources in their lexical form as *terms*;
//! - Triples and quads;
//! - Interpretations projecting resources from the lexical domain to the value
//!   domain;
//! - Graphs and datasets representing collections of interpreted triples/quads.
//!
//! # Getting started
//!
//! Terms, triples and quads are built with the [`triple!`] and [`quad!`]
//! macros, which accept the same lexical syntax as N-Triples/N-Quads:
//!
//! ```
//! use rdfx::{LocalTerm, triple};
//!
//! // RDF 1.1: a language-tagged literal in object position.
//! let t = triple!(<"http://example.org/alice"> <"http://xmlns.com/foaf/0.1/name"> "Alice" @ "en");
//!
//! // RDF 1.2: a triple term as the object.
//! let annotated = triple!(
//!     <"http://example.org/bob"> <"http://example.org/says">
//!     <<( <"http://example.org/s"> <"http://example.org/p"> <"http://example.org/o"> )>>
//! );
//! assert!(matches!(annotated.into_object(), LocalTerm::Triple(_)));
//! ```
//!
//! Storage is uniform over a single resource type:
//! [`BTreeDataset<R>`](dataset::BTreeDataset),
//! [`HashDataset<R>`](dataset::HashDataset) and their indexed variants hold
//! the same `R: Resource` in every position. [`LocalTerm`] is deliberately *not* a [`Resource`] —
//! a literal cannot be a subject — so store interned handles from a
//! [`vocabulary`], or wrap the lexical term in a newtype opted in with
//! [`impl_resource!`]:
//!
//! ```
//! use rdfx::{Quad, dataset::{BTreeDataset, TraversableDataset}};
//!
//! let mut dataset: BTreeDataset<usize> = BTreeDataset::new();
//! dataset.insert(Quad(1, 0, 2, None));
//! dataset.insert(Quad(2, 0, 3, Some(9)));
//!
//! assert_eq!(dataset.quads_count(), 2);
//! ```
//!
//! Blank node isomorphism lives in [`dataset::isomorphism`]: the plain
//! entry points answer yes or no, while
//! [`find_bijection_with`](dataset::isomorphism::find_bijection_with) returns
//! the blank node mapping. The `_with` variants take an
//! [`Interpretation`](interpretation::Interpretation), which is what makes
//! blank nodes and triple-term interiors visible to the matcher; without one
//! resources are opaque and equivalence degrades to set equality.
//!
//! ```
//! use rdfx::{Quad, dataset::{BTreeDataset, isomorphism::dataset_equivalent}};
//!
//! let mut a: BTreeDataset<usize> = BTreeDataset::new();
//! a.insert(Quad(1, 0, 2, None));
//!
//! let mut b: BTreeDataset<usize> = BTreeDataset::new();
//! b.insert(Quad(1, 0, 2, None));
//!
//! assert!(dataset_equivalent(&a, &b));
//! ```
//!
//! # RDF 1.2
//!
//! This crate targets [RDF 1.2 Concepts][rdf12] alongside RDF 1.1 — RDF 1.1
//! graphs are a syntactic subset (they never construct the new variants):
//!
//! - **Triple terms** ([RDF 1.2 §4][s32-tt]) — a triple appearing as an RDF
//!   term, restricted to object position in strict RDF (any position via the
//!   [`GeneralizedTriple`] / [`GeneralizedLocalTerm`] types). Carried by
//!   [`LocalTerm::Triple`]; aliased as [`TripleTerm`]. Macro syntax
//!   `<<( s p o )>>`.
//! - **Directional language strings** ([RDF 1.2 §3.4][s32-dls]) — language
//!   tag plus base direction (`ltr` / `rtl`), datatype `rdf:dirLangString`.
//!   Carried by [`LiteralType::DirLangString`]; constructor
//!   [`Literal::dir_lang_string`]; helper enum [`Direction`]. Macro syntax
//!   `"v" @ "lang" / "ltr"`.
//! - **`rdf:reifies` annotation model** ([RDF 1.2 §6][s32-ann]) — a
//!   round-trippable mapping between triple-term-bearing graphs and their
//!   reified RDF 1.1-compatible form, exposed as
//!   [`star::unstar_graph`] / [`star::restar_graph`] (and the dataset
//!   variants).
//!
//! ```
//! use rdfx::{Direction, LiteralType, LocalTerm, Term, triple};
//!
//! let t = triple!(<"http://example.org/doc"> <"http://example.org/title"> "hello" @ "ar" / "rtl");
//! let LocalTerm::Named(Term::Literal(literal)) = t.into_object() else { unreachable!() };
//!
//! assert!(matches!(literal.into_type(), LiteralType::DirLangString { direction: Direction::Rtl, .. }));
//! ```
//!
//! # Feature flags
//!
//! All features are off by default.
//!
//! - `serde` — `Serialize` / `Deserialize` for terms, literals, triples and
//!   quads.
//! - `meta` — the [`meta`] module: [`locspan`]-tagged values and the
//!   [`Strip`](meta::Strip) trait.
//! - `contextual` — `Display` through a vocabulary via the
//!   [`contextual`](https://crates.io/crates/contextual) crate.
//! - `uuid-generator`, `uuid-generator-v3`, `uuid-generator-v4`,
//!   `uuid-generator-v5` — UUID-based blank node generators.
//!
//! [rdf]:    <https://w3c.github.io/rdf-primer/spec/>
//! [w3c]:    <https://www.w3.org/>
//! [rdf12]:  <https://www.w3.org/TR/rdf12-concepts/>
//! [s32-tt]:  <https://www.w3.org/TR/rdf12-concepts/#section-triples>
//! [s32-dls]: <https://www.w3.org/TR/rdf12-concepts/#section-Graph-Literal>
//! [s32-ann]: <https://www.w3.org/TR/rdf12-concepts/#section-reification>
#![recursion_limit = "1024"]

#[doc(hidden)]
pub use iri_rs;

#[doc(hidden)]
pub use langtag;

mod blankid;
mod display;
mod generalized;
mod id;
mod literal;
mod r#macro;
mod positions;
mod quad;
mod schema;
mod swar;
mod term;
mod triple;

pub use blankid::*;
pub use display::*;
pub use generalized::*;
pub use id::*;
pub use literal::*;
#[doc(hidden)]
pub use positions::__seal;
pub use positions::{IsGraph, IsObject, IsPredicate, IsSubject, Resource};
pub use quad::*;
pub use schema::*;
pub use term::*;
pub use triple::*;

pub mod dataset;
pub mod interpretation;
pub mod pattern;
pub mod star;
pub mod utils;
pub mod vocabulary;

#[cfg(feature = "meta")]
pub mod meta;

pub use dataset::Dataset;
pub use interpretation::{Interpretation, InterpretationMut};
pub use iri_rs::{Iri, IriBuf, iri};

pub const XSD_STRING: Iri<&'static str> = iri!("http://www.w3.org/2001/XMLSchema#string");
