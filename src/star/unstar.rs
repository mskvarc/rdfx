//! Unstar mapping: triple-term-bearing graphs → reified RDF 1.1 form.
//!
//! Implements the canonical N-Triples 1.2 → RDF 1.1 rewrite: every
//! triple-term occurrence is replaced by a fresh reifier blank node, with
//! the four reification triples
//!
//! ```text
//! _:r rdf:type      rdf:TripleTerm .
//! _:r rdf:subject   <s> .
//! _:r rdf:predicate <p> .
//! _:r rdf:object    <o> .
//! ```
//!
//! emitted alongside the rewritten triple.
//!
//! ## `rdf:reifies` annotations (RDF 1.2 §6)
//!
//! When the input already contains explicit reifier annotations of the form
//!
//! ```text
//! _:r rdf:reifies <<( s p o )>> .
//! ```
//!
//! the existing reifier identity (`_:r`) is preserved: the four-tuple is
//! emitted with `_:r` as subject, the original `rdf:reifies` triple is
//! dropped, and any other triples about `_:r` (the annotations themselves)
//! flow through untouched. Per RDF 1.2 §6 the reifier may be a blank node or
//! an IRI; both are honored.

use std::collections::{HashMap, HashSet};

use iri_rs::IriBuf;

use crate::{
    BlankIdBuf,
    Id,
    LocalGenerator,
    LocalTerm,
    Quad,
    RDF_OBJECT,
    RDF_PREDICATE,
    RDF_REIFIES,
    RDF_SUBJECT,
    RDF_TRIPLE_TERM,
    RDF_TYPE,
    Term,
    Triple,
    TripleTerm,
    dataset::{DatasetMut, GraphMut, TraversableDataset, TraversableGraph},
};

/// Options for [`unstar_graph`] / [`unstar_dataset`].
#[derive(Debug, Clone, Copy)]
pub struct UnstarOptions {
    /// If `true`, identical triple-term occurrences (compared by value) share
    /// a single reifier blank node. If `false`, each occurrence (other than
    /// those already named via an explicit `rdf:reifies` annotation) gets a
    /// fresh reifier.
    ///
    /// Reifiers carried over from explicit `rdf:reifies` annotations are
    /// always honored regardless of this flag.
    pub reuse_reifier: bool,
}

impl Default for UnstarOptions {
    fn default() -> Self {
        Self { reuse_reifier: true }
    }
}

/// Unfolds every triple-term object in `src` into the reified RDF 1.1 form,
/// inserting the result (re-projected triples + reification triples) into
/// `dest`. Fresh reifier blank ids come from `generator`. Existing
/// `rdf:reifies` annotations are recognised; the named reifier is reused.
pub fn unstar_graph<G, M, Gen>(src: &G, dest: &mut M, generator: &mut Gen, opts: UnstarOptions)
where
    G: TraversableGraph<Subject = Id, Predicate = IriBuf, Object = LocalTerm>,
    M: GraphMut<Subject = Id, Predicate = IriBuf, Object = LocalTerm>,
    Gen: LocalGenerator,
{
    let mut cache: HashMap<TripleTerm, Id> = HashMap::new();
    let mut emitted: HashSet<Id> = HashSet::new();

    // Pre-pass: populate cache from `_:r rdf:reifies <<...>>` annotations.
    for t in src.triples() {
        if let Some(body) = reifies_body(t.1, t.2) {
            cache.entry(body).or_insert_with(|| t.0.clone());
        }
    }

    for t in src.triples() {
        if reifies_body(t.1, t.2).is_some() {
            // Drop the rdf:reifies triple itself; emit the four-tuple via
            // the body recursion using the cached reifier.
            let _ = unstar_object_inner(t.2.clone(), &mut |q| dest.insert(q), generator, &mut cache, &mut emitted, opts);
            continue;
        }
        let s = t.0.clone();
        let p = t.1.clone();
        let o = unstar_object_inner(t.2.clone(), &mut |q| dest.insert(q), generator, &mut cache, &mut emitted, opts);
        dest.insert(Triple(s, p, o));
    }
}

/// Per-graph reification state: (body→reifier cache, emitted-reifier set).
type GraphReifyState = (HashMap<TripleTerm, Id>, HashSet<Id>);

/// Dataset-flavoured [`unstar_graph`]. Reification triples are emitted into
/// the same graph as their originating quad. `rdf:reifies` annotations are
/// recognised within each graph context.
pub fn unstar_dataset<D, M, Gen>(src: &D, dest: &mut M, generator: &mut Gen, opts: UnstarOptions)
where
    D: TraversableDataset<Subject = Id, Predicate = IriBuf, Object = LocalTerm, Graph = Id>,
    M: DatasetMut<Subject = Id, Predicate = IriBuf, Object = LocalTerm, Graph = Id>,
    Gen: LocalGenerator,
{
    // Cache and emitted set are keyed per graph: reifier identity is local
    // to its graph.
    let mut per_graph: HashMap<Option<Id>, GraphReifyState> = HashMap::new();

    for q in src.quads() {
        let g = q.3.cloned();
        if let Some(body) = reifies_body(q.1, q.2) {
            let (cache, _) = per_graph.entry(g).or_default();
            cache.entry(body).or_insert_with(|| q.0.clone());
        }
    }

    for q in src.quads() {
        let g = q.3.cloned();
        let (cache, emitted) = per_graph.entry(g.clone()).or_default();
        if reifies_body(q.1, q.2).is_some() {
            let g2 = g.clone();
            let _ = unstar_object_inner(
                q.2.clone(),
                &mut |Triple(rs, rp, ro)| dest.insert(Quad(rs, rp, ro, g2.clone())),
                generator,
                cache,
                emitted,
                opts,
            );
            continue;
        }
        let s = q.0.clone();
        let p = q.1.clone();
        let g2 = g.clone();
        let o = unstar_object_inner(
            q.2.clone(),
            &mut |Triple(rs, rp, ro)| dest.insert(Quad(rs, rp, ro, g2.clone())),
            generator,
            cache,
            emitted,
            opts,
        );
        dest.insert(Quad(s, p, o, g));
    }
}

/// Detects `rdf:reifies` triples with a triple-term object; returns the
/// body if matched.
fn reifies_body(p: &IriBuf, o: &LocalTerm) -> Option<TripleTerm> {
    if p.as_str() != RDF_REIFIES.as_str() {
        return None;
    }
    match o {
        LocalTerm::Triple(b) => Some((**b).clone()),
        _ => None,
    }
}

fn unstar_object_inner<F, Gen>(
    o: LocalTerm,
    emit: &mut F,
    generator: &mut Gen,
    cache: &mut HashMap<TripleTerm, Id>,
    emitted: &mut HashSet<Id>,
    opts: UnstarOptions,
) -> LocalTerm
where
    F: FnMut(Triple<Id, IriBuf, LocalTerm>),
    Gen: LocalGenerator,
{
    match o {
        LocalTerm::Triple(boxed) => {
            let t = std::sync::Arc::try_unwrap(boxed).unwrap_or_else(|arc| (*arc).clone());

            // Recurse on inner object first so the cache is populated for
            // nested triple terms before the outer four-tuple is emitted.
            let inner_o = unstar_object_inner(t.2.clone(), emit, generator, cache, emitted, opts);
            let inner_s = id_to_local_term(t.0.clone());
            let inner_p = LocalTerm::iri(t.1.clone());

            // Reifier resolution:
            //   1. cache hit (either pre-pass or earlier reuse) → use that
            //      reifier, regardless of `reuse_reifier`.
            //   2. `reuse_reifier=true` → mint, insert into cache.
            //   3. `reuse_reifier=false` → mint, do not insert.
            let reifier: Id = if let Some(existing) = cache.get(&t) {
                existing.clone()
            } else {
                let fresh = Id::BlankId(next_blank(generator));
                if opts.reuse_reifier {
                    cache.insert(t, fresh.clone());
                }
                fresh
            };

            if emitted.insert(reifier.clone()) {
                emit(Triple(reifier.clone(), IriBuf::from(RDF_TYPE), LocalTerm::iri(IriBuf::from(RDF_TRIPLE_TERM))));
                emit(Triple(reifier.clone(), IriBuf::from(RDF_SUBJECT), inner_s));
                emit(Triple(reifier.clone(), IriBuf::from(RDF_PREDICATE), inner_p));
                emit(Triple(reifier.clone(), IriBuf::from(RDF_OBJECT), inner_o));
            }
            id_to_local_term(reifier)
        }
        other => other,
    }
}

#[allow(clippy::panic)]
fn next_blank<G: LocalGenerator>(generator: &mut G) -> BlankIdBuf {
    // The reification mapping requires fresh blank node identifiers as
    // reifiers; the [`LocalGenerator`] contract for this helper is that
    // `next_local_term` yields a `LocalTerm::BlankId`. The standard
    // [`Blank`](crate::generator::Blank) generator satisfies this; bespoke
    // generators yielding non-blank terms violate the contract and panic.
    match generator.next_local_term() {
        LocalTerm::BlankId(b) => b,
        other => panic!("LocalGenerator yielded non-blank LocalTerm during reification: {other:?}"),
    }
}

fn id_to_local_term(id: Id) -> LocalTerm {
    match id {
        Id::Iri(i) => LocalTerm::Named(Term::Iri(i)),
        Id::BlankId(b) => LocalTerm::BlankId(b),
    }
}
