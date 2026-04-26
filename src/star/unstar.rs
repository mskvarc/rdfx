//! Unstar mapping: triple-term-bearing graphs → reified RDF 1.1 form.

use std::collections::HashMap;

use iri_rs::IriBuf;

use crate::{
    BlankIdBuf, Id, LocalGenerator, LocalTerm, Quad, RDF_OBJECT, RDF_PREDICATE, RDF_SUBJECT, RDF_TRIPLE_TERM, RDF_TYPE, Term, Triple, TripleTerm,
    dataset::{DatasetMut, GraphMut, TraversableDataset, TraversableGraph},
};

/// Options for [`unstar_graph`] / [`unstar_dataset`].
#[derive(Debug, Clone, Copy)]
pub struct UnstarOptions {
    /// If `true`, identical triple-term occurrences (compared by value) share
    /// a single reifier blank node. If `false`, each occurrence gets a fresh
    /// reifier.
    pub reuse_reifier: bool,
}

impl Default for UnstarOptions {
    fn default() -> Self {
        Self { reuse_reifier: true }
    }
}

/// Unfolds every triple-term object in `src` into the reified RDF 1.1 form,
/// inserting the result (re-projected triples + reification triples) into
/// `dest`. Fresh reifier blank ids come from `generator`.
pub fn unstar_graph<G, M, Gen>(src: &G, dest: &mut M, generator: &mut Gen, opts: UnstarOptions)
where
    G: TraversableGraph<Subject = Id, Predicate = IriBuf, Object = LocalTerm>,
    M: GraphMut<Subject = Id, Predicate = IriBuf, Object = LocalTerm>,
    Gen: LocalGenerator,
{
    let mut cache: HashMap<TripleTerm, BlankIdBuf> = HashMap::new();
    for t in src.triples() {
        let s = t.0.clone();
        let p = t.1.clone();
        let o = unstar_object(t.2.clone(), |q| {
            let Triple(rs, rp, ro) = q;
            dest.insert(Triple(rs, rp, ro))
        }, generator, &mut cache, opts);
        dest.insert(Triple(s, p, o));
    }
}

/// Dataset-flavoured [`unstar_graph`]. Reification triples are emitted into
/// the same graph as their originating quad.
pub fn unstar_dataset<D, M, Gen>(src: &D, dest: &mut M, generator: &mut Gen, opts: UnstarOptions)
where
    D: TraversableDataset<Subject = Id, Predicate = IriBuf, Object = LocalTerm, Graph = Id>,
    M: DatasetMut<Subject = Id, Predicate = IriBuf, Object = LocalTerm, Graph = Id>,
    Gen: LocalGenerator,
{
    let mut cache: HashMap<TripleTerm, BlankIdBuf> = HashMap::new();
    for q in src.quads() {
        let s = q.0.clone();
        let p = q.1.clone();
        let g = q.3.cloned();
        let o = unstar_object(
            q.2.clone(),
            |Triple(rs, rp, ro)| dest.insert(Quad(rs, rp, ro, g.clone())),
            generator,
            &mut cache,
            opts,
        );
        dest.insert(Quad(s, p, o, g));
    }
}

fn unstar_object<F, Gen>(o: LocalTerm, mut emit: F, generator: &mut Gen, cache: &mut HashMap<TripleTerm, BlankIdBuf>, opts: UnstarOptions) -> LocalTerm
where
    F: FnMut(Triple<Id, IriBuf, LocalTerm>),
    Gen: LocalGenerator,
{
    unstar_object_inner(o, &mut emit, generator, cache, opts)
}

fn unstar_object_inner<F, Gen>(o: LocalTerm, emit: &mut F, generator: &mut Gen, cache: &mut HashMap<TripleTerm, BlankIdBuf>, opts: UnstarOptions) -> LocalTerm
where
    F: FnMut(Triple<Id, IriBuf, LocalTerm>),
    Gen: LocalGenerator,
{
    match o {
        LocalTerm::Triple(boxed) => {
            let t = *boxed;

            // Recurse on inner object first so cache reflects nested
            // reifier reuse correctly.
            let inner_o = unstar_object_inner(t.2.clone(), emit, generator, cache, opts);
            let inner_s = id_to_local_term(t.0.clone());
            let inner_p = LocalTerm::iri(t.1.clone());

            let reifier = if opts.reuse_reifier {
                if let Some(b) = cache.get(&t) {
                    return LocalTerm::BlankId(b.clone());
                }
                let b = next_blank(generator);
                cache.insert(t, b.clone());
                b
            } else {
                next_blank(generator)
            };

            let r_id = Id::BlankId(reifier.clone());
            emit(Triple(r_id.clone(), IriBuf::from(RDF_TYPE), LocalTerm::iri(IriBuf::from(RDF_TRIPLE_TERM))));
            emit(Triple(r_id.clone(), IriBuf::from(RDF_SUBJECT), inner_s));
            emit(Triple(r_id.clone(), IriBuf::from(RDF_PREDICATE), inner_p));
            emit(Triple(r_id, IriBuf::from(RDF_OBJECT), inner_o));
            LocalTerm::BlankId(reifier)
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
