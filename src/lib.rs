//! The [Resource Description Framework (RDF)][rdf] is a very simple graph data
//! model defined by the [World Wide Web Consortium (W3C)][w3c] to represent
//! arbitrary pieces of information, primarily intended for the web. Nodes of
//! the graph are called *resources*, and resources are connected together using
//! *relations*, which are resources themselves.
//!
//! This is a utility library providing common types, data-structures, traits,
//! constants and macro definitions to deal with RDF data:
//! - IRIs (through the `iref` crate), blank node identifiers and literals to
//!   represent resources in their lexical form as *terms*;
//! - Triples and quads;
//! - Interpretations projecting resources from the lexical domain to the value
//!   domain;
//! - Graphs and datasets representing collections of interpreted triples/quads.
//!
//! # RDF 1.2
//!
//! As of `0.24.0` this crate targets [RDF 1.2 Concepts][rdf12] alongside
//! RDF 1.1 — RDF 1.1 graphs are a syntactic subset (they never construct the
//! new variants):
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

#[cfg(feature = "meta")]
pub mod meta;

pub use dataset::Dataset;
pub use interpretation::{Interpretation, InterpretationMut};
pub use iri_rs::{Iri, IriBuf, iri};

pub const XSD_STRING: Iri<&'static str> = iri!("http://www.w3.org/2001/XMLSchema#string");
