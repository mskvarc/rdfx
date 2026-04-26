//! Restar mapping: reified RDF 1.1 form → triple-term-bearing graphs.
//!
//! Detects subgraphs of the form
//!
//! ```text
//! _:r rdf:type      rdf:TripleTerm .
//! _:r rdf:subject   <s> .
//! _:r rdf:predicate <p> .
//! _:r rdf:object    <o> .
//! ```
//!
//! and refolds occurrences of `_:r` in object position back into
//! [`LocalTerm::Triple`](crate::LocalTerm::Triple) terms. Reifier blanks
//! whose four reification triples are not all present are left untouched.

use std::collections::{HashMap, HashSet};

use iri_rs::IriBuf;

use crate::{
    BlankIdBuf, Id, LocalTerm, Quad, RDF_OBJECT, RDF_PREDICATE, RDF_SUBJECT, RDF_TRIPLE_TERM, RDF_TYPE, Term, Triple, TripleTerm,
    dataset::{DatasetMut, GraphMut, TraversableDataset, TraversableGraph},
};

/// Refolds reified triple-term subgraphs in `src` back into triple terms,
/// inserting the result into `dest`. Reification triples for refolded
/// reifiers are dropped; everything else is copied through, with reifier
/// blank ids in object position replaced by the corresponding refolded
/// triple term.
pub fn restar_graph<G, M>(src: &G, dest: &mut M)
where
    G: TraversableGraph<Subject = Id, Predicate = IriBuf, Object = LocalTerm>,
    M: GraphMut<Subject = Id, Predicate = IriBuf, Object = LocalTerm>,
{
    let mut typed: HashSet<BlankIdBuf> = HashSet::new();
    let mut parts: HashMap<BlankIdBuf, Parts> = HashMap::new();

    for t in src.triples() {
        scan_triple(t.0, t.1, t.2, &mut typed, &mut parts);
    }

    let expanded = expand_all(&typed, &parts);

    for t in src.triples() {
        if is_reification_triple(t.0, t.1, t.2, &expanded) {
            continue;
        }
        let new_o = replace_reifier(t.2.clone(), &expanded);
        dest.insert(Triple(t.0.clone(), t.1.clone(), new_o));
    }
}

/// Dataset-flavoured [`restar_graph`].
pub fn restar_dataset<D, M>(src: &D, dest: &mut M)
where
    D: TraversableDataset<Subject = Id, Predicate = IriBuf, Object = LocalTerm, Graph = Id>,
    M: DatasetMut<Subject = Id, Predicate = IriBuf, Object = LocalTerm, Graph = Id>,
{
    let mut typed: HashSet<BlankIdBuf> = HashSet::new();
    let mut parts: HashMap<BlankIdBuf, Parts> = HashMap::new();

    for q in src.quads() {
        scan_triple(q.0, q.1, q.2, &mut typed, &mut parts);
    }

    let expanded = expand_all(&typed, &parts);

    for q in src.quads() {
        if is_reification_triple(q.0, q.1, q.2, &expanded) {
            continue;
        }
        let new_o = replace_reifier(q.2.clone(), &expanded);
        dest.insert(Quad(q.0.clone(), q.1.clone(), new_o, q.3.cloned()));
    }
}

#[derive(Default, Clone)]
struct Parts {
    subject: Option<LocalTerm>,
    predicate: Option<LocalTerm>,
    object: Option<LocalTerm>,
}

fn scan_triple(s: &Id, p: &IriBuf, o: &LocalTerm, typed: &mut HashSet<BlankIdBuf>, parts: &mut HashMap<BlankIdBuf, Parts>) {
    let Id::BlankId(b) = s else { return };
    let p_str = p.as_str();
    if p_str == RDF_TYPE.as_str() {
        if let Some(iri) = o.as_iri() {
            if iri.as_str() == RDF_TRIPLE_TERM.as_str() {
                typed.insert(b.clone());
            }
        }
    } else if p_str == RDF_SUBJECT.as_str() {
        parts.entry(b.clone()).or_default().subject = Some(o.clone());
    } else if p_str == RDF_PREDICATE.as_str() {
        parts.entry(b.clone()).or_default().predicate = Some(o.clone());
    } else if p_str == RDF_OBJECT.as_str() {
        parts.entry(b.clone()).or_default().object = Some(o.clone());
    }
}

/// Expand all valid reifiers (typed + complete quartet) into refolded
/// triple terms. Recursively unfolds nested reifiers in object position.
fn expand_all(typed: &HashSet<BlankIdBuf>, parts: &HashMap<BlankIdBuf, Parts>) -> HashMap<BlankIdBuf, TripleTerm> {
    // Build raw triples (subject Id, predicate IriBuf, object LocalTerm) per
    // reifier that has all parts and a valid subject/predicate kind.
    let mut raw: HashMap<BlankIdBuf, TripleTerm> = HashMap::new();
    for b in typed {
        if let Some(p) = parts.get(b) {
            if let (Some(s), Some(pred), Some(o)) = (&p.subject, &p.predicate, &p.object) {
                if let (Some(sid), Some(piri)) = (local_term_to_id(s), local_term_to_iri(pred)) {
                    raw.insert(b.clone(), Triple(sid, piri, o.clone()));
                }
            }
        }
    }

    let mut expanded: HashMap<BlankIdBuf, TripleTerm> = HashMap::new();
    let mut visiting: HashSet<BlankIdBuf> = HashSet::new();
    let keys: Vec<BlankIdBuf> = raw.keys().cloned().collect();
    for b in &keys {
        expand_one(b, &raw, &mut expanded, &mut visiting);
    }
    expanded
}

fn expand_one(b: &BlankIdBuf, raw: &HashMap<BlankIdBuf, TripleTerm>, expanded: &mut HashMap<BlankIdBuf, TripleTerm>, visiting: &mut HashSet<BlankIdBuf>) {
    if expanded.contains_key(b) {
        return;
    }
    if !visiting.insert(b.clone()) {
        // Cycle — leave unfolded.
        return;
    }
    if let Some(t) = raw.get(b) {
        let new_o = match &t.2 {
            LocalTerm::BlankId(child) if raw.contains_key(child) => {
                expand_one(child, raw, expanded, visiting);
                match expanded.get(child) {
                    Some(child_t) => LocalTerm::triple(child_t.clone()),
                    None => t.2.clone(),
                }
            }
            other => other.clone(),
        };
        expanded.insert(b.clone(), Triple(t.0.clone(), t.1.clone(), new_o));
    }
    visiting.remove(b);
}

fn is_reification_triple(s: &Id, p: &IriBuf, o: &LocalTerm, expanded: &HashMap<BlankIdBuf, TripleTerm>) -> bool {
    let Id::BlankId(b) = s else { return false };
    if !expanded.contains_key(b) {
        return false;
    }
    let p_str = p.as_str();
    if p_str == RDF_TYPE.as_str() {
        if let Some(iri) = o.as_iri() {
            return iri.as_str() == RDF_TRIPLE_TERM.as_str();
        }
        return false;
    }
    p_str == RDF_SUBJECT.as_str() || p_str == RDF_PREDICATE.as_str() || p_str == RDF_OBJECT.as_str()
}

fn replace_reifier(o: LocalTerm, expanded: &HashMap<BlankIdBuf, TripleTerm>) -> LocalTerm {
    match o {
        LocalTerm::BlankId(ref b) if expanded.contains_key(b) => LocalTerm::triple(expanded[b].clone()),
        other => other,
    }
}

fn local_term_to_id(t: &LocalTerm) -> Option<Id> {
    match t {
        LocalTerm::Named(Term::Iri(iri)) => Some(Id::Iri(iri.clone())),
        LocalTerm::BlankId(b) => Some(Id::BlankId(b.clone())),
        _ => None,
    }
}

fn local_term_to_iri(t: &LocalTerm) -> Option<IriBuf> {
    match t {
        LocalTerm::Named(Term::Iri(iri)) => Some(iri.clone()),
        _ => None,
    }
}
