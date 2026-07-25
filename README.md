# rdfx

[![Crate](https://img.shields.io/crates/v/rdfx.svg?style=flat-square)](https://crates.io/crates/rdfx)
[![Docs](https://img.shields.io/docsrs/rdfx?style=flat-square)](https://docs.rs/rdfx)
[![MSRV](https://img.shields.io/crates/msrv/rdfx?style=flat-square)](https://crates.io/crates/rdfx)
[![License](https://img.shields.io/crates/l/rdfx.svg?style=flat-square)](#license)

Core [RDF 1.2](https://www.w3.org/TR/rdf12-concepts/) data structures for Rust — terms (triple terms included), triples and quads, graphs and datasets, interpretations and vocabularies, blank node isomorphism, and `rdf:reifies` round-tripping.

```rust
use rdfx::{LocalTerm, triple};

// RDF 1.1: language-tagged literal in object position.
let t = triple!(<"http://example.org/alice"> <"http://xmlns.com/foaf/0.1/name"> "Alice" @ "en");

// RDF 1.2: a triple term as the object (`<<( s p o )>>`).
let annotated = triple!(
    <"http://example.org/bob"> <"http://example.org/says">
    <<( <"http://example.org/s"> <"http://example.org/p"> <"http://example.org/o"> )>>
);
assert!(matches!(annotated.into_object(), LocalTerm::Triple(_)));
```

Graphs and datasets are generic over one resource type, so pick the representation that fits — indices interned in a vocabulary, or any type opted in with `impl_resource!`:

```rust
use rdfx::{Quad, dataset::{BTreeDataset, TraversableDataset}};

let mut dataset: BTreeDataset<usize> = BTreeDataset::new();
dataset.insert(Quad(1, 0, 2, None));
dataset.insert(Quad(2, 0, 3, Some(9)));

assert_eq!(dataset.quads_count(), 2);
```

---

## Why this fork

Fork of [`rdf-types`](https://crates.io/crates/rdf-types) by [Timothée Haudebourg](https://github.com/timothee-haudebourg/rdf-types). The RDF 1.1 model carries over unchanged; this fork adds:

- RDF 1.2: triple terms, directional language strings (`rdf:dirLangString`), and the `rdf:reifies` annotation model with `unstar` / `restar` round-tripping.
- Hash-backed graph and dataset implementations plus indexed variants, alongside the upstream B-tree ones, and a graph/dataset trait split so traversal, pattern matching and mutation are separate bounds.
- Rewritten isomorphism: bitmap signature refinement, candidate ordering by set size, augmenting-path bipartite matching instead of greedy matching, and triple-term descent. Two correctness fixes came with it — false positives on blank-free quads, and stale subject/predicate/object index sets after removal.
- Sealed position-marker traits (`IsSubject`, `IsPredicate`, `IsObject`, `IsGraph`) turning RDF position constraints into compile-time errors, with generalized-RDF types for when you need to lift them.
- Performance: SWAR blank id validation, `memchr`-accelerated `Display`, `itoa` and `uuid` fast paths in blank node generators, allocation removal across isomorphism.
- Rust 2024 edition, MSRV 1.85, single crate instead of a workspace.
- Renamed crate: `rdf-types` → `rdfx`. Depends on [`iri-rs`](https://crates.io/crates/iri-rs) (fork of `iref`) instead of `iref`, which is the point of the fork.
- Criterion bench suite (`dataset`, `isomorphism`, `swar`, `triple_terms`, `vocabulary`).

Credit and history preserved — see [Attribution](#attribution).

## Install

```sh
cargo add rdfx
```

## Feature flags

| Flag                | Default | Enables                                                                               |
| ------------------- | :-----: | ------------------------------------------------------------------------------------- |
| `serde`             |         | `Serialize` / `Deserialize` for terms, literals, triples and quads                     |
| `meta`              |         | `meta` module: [`locspan`](https://crates.io/crates/locspan)-tagged values and `Strip` |
| `contextual`        |         | `Display` through a vocabulary via [`contextual`](https://crates.io/crates/contextual) |
| `uuid-generator`    |         | All UUID blank node generators (v3, v4, v5)                                            |
| `uuid-generator-v3` |         | Name-based (MD5) UUID blank node generator                                             |
| `uuid-generator-v4` |         | Random UUID blank node generator                                                       |
| `uuid-generator-v5` |         | Name-based (SHA-1) UUID blank node generator                                           |

## RDF 1.2

RDF 1.1 data is a syntactic subset — the new variants are simply never constructed. Three additions matter.

**Triple terms.** A triple used as a term. Strict RDF admits them in object position only; the generalized types lift that restriction. Macro syntax: `<<( s p o )>>`.

**Directional language strings.** A language tag plus base direction, datatype `rdf:dirLangString`. Macro syntax: `"value" @ "lang" / "rtl"`.

```rust
use rdfx::{Direction, LiteralType, LocalTerm, Term, triple};

let t = triple!(<"http://example.org/doc"> <"http://example.org/title"> "hello" @ "ar" / "rtl");
let LocalTerm::Named(Term::Literal(literal)) = t.into_object() else { unreachable!() };

assert!(matches!(literal.into_type(), LiteralType::DirLangString { direction: Direction::Rtl, .. }));
```

**The `rdf:reifies` annotation model.** `star::unstar_graph` rewrites triple terms into RDF 1.1-compatible reified form; `star::restar_graph` folds them back. The pair round-trips, so 1.2 data survives a 1.1-only channel.

## Storage

Storage is uniform: `BTreeDataset<R>`, `HashDataset<R>` and their indexed variants hold one `R: Resource` in every position. `LocalTerm` — the lexical alphabet — is deliberately *not* a `Resource`, because a literal cannot be a subject. Two ways around that:

- Intern lexical values in a vocabulary and store the handles. Comparison and cloning collapse to integer operations.
- Wrap the lexical term in a newtype, opt it in with `impl_resource!`, and supply an `Interpretation` mapping the wrapper back to IRIs, literals and triple-term components.

Indexed variants carry subject/predicate/object indices for pattern matching; plain variants are smaller and build faster.

## Isomorphism

Two RDF graphs are isomorphic when some bijection over blank nodes makes them equal. `graph_equivalent` / `dataset_equivalent` answer yes or no; `find_bijection_with` returns the mapping itself.

```rust
use rdfx::{Quad, dataset::{BTreeDataset, isomorphism::dataset_equivalent}};

let mut a: BTreeDataset<usize> = BTreeDataset::new();
a.insert(Quad(1, 0, 2, None));

let mut b: BTreeDataset<usize> = BTreeDataset::new();
b.insert(Quad(1, 0, 2, None));

assert!(dataset_equivalent(&a, &b));
```

The `_with` variants take an `Interpretation`, which is what makes blank nodes — and the interiors of triple terms — visible to the matcher. Without one, resources are opaque and equivalence degrades to set equality.

## Vocabularies

A vocabulary maps lexical values onto lightweight handles, so a dataset can store integers instead of IRIs:

```rust
use rdfx::{iri, vocabulary::{IndexVocabulary, IriVocabulary, IriVocabularyMut}};

let mut vocabulary: IndexVocabulary = IndexVocabulary::default();
let i = vocabulary.insert(iri!("http://example.org/s"));

assert_eq!(vocabulary.iri(&i), Some(iri!("http://example.org/s")));
```

## Benchmarks

```sh
cargo bench
```

Criterion output: `target/criterion/`.

## MSRV

Rust 1.85 (edition 2024).

## Attribution

Original crate: [`rdf-types`](https://crates.io/crates/rdf-types) by [Timothée Haudebourg](https://github.com/timothee-haudebourg/rdf-types). Upstream commits are preserved in this repo's history under their original authorship. This fork is RDF 1.2 support, extra storage backends, a rewritten isomorphism engine, and a layer of performance work on top of the original design.

## License

Dual-licensed, same as upstream. Pick whichever fits:

- [Apache-2.0](https://github.com/mskvarc/rdfx/blob/master/LICENSE-APACHE.md)
- [MIT](https://github.com/mskvarc/rdfx/blob/master/LICENSE-MIT.md)
</content>
