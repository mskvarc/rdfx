#![cfg(feature = "meta")]

use rdf_term::{
    BlankIdBuf,
    LocalTerm,
    Quad,
    dataset::{BTreeDataset, HashDataset},
    meta::{Meta, MetaQuad, MetaQuadFull, Strip},
};

fn blank_term(name: &str) -> LocalTerm {
    LocalTerm::Anonymous(BlankIdBuf::new(name.to_string()).unwrap())
}

fn iri_term(path: &str) -> LocalTerm {
    let s = format!("http://example.com/{path}");
    LocalTerm::iri(iri_rs::IriBuf::new(s).unwrap())
}

#[test]
fn strip_meta_quad_collapses_to_plain_quad() {
    let s = blank_term("_:a");
    let p = iri_term("p");
    let o = iri_term("o");

    let mq: MetaQuad<u32> = Meta(Quad(Meta(s.clone(), 1), Meta(p.clone(), 2), Meta(o.clone(), 3), None), 7);

    let stripped: Quad<LocalTerm> = mq.strip();
    assert_eq!(stripped, Quad(s, p, o, None));
}

#[test]
fn btree_dataset_of_meta_localterm_inserts() {
    let mut d: BTreeDataset<Meta<LocalTerm, u32>> = BTreeDataset::new();
    d.insert(Quad(Meta(blank_term("_:a"), 1), Meta(iri_term("p"), 1), Meta(iri_term("o"), 1), None));
    assert_eq!(d.len(), 1);
}

#[test]
fn meta_eq_distinguishes_metadata() {
    let s = blank_term("_:a");
    let p = iri_term("p");
    let o = iri_term("o");

    let mut d: BTreeDataset<Meta<LocalTerm, u32>> = BTreeDataset::new();
    d.insert(Quad(Meta(s.clone(), 1), Meta(p.clone(), 1), Meta(o.clone(), 1), None));
    d.insert(Quad(Meta(s, 2), Meta(p, 2), Meta(o, 2), None));
    assert_eq!(d.len(), 2);
}

#[test]
fn hash_dataset_of_meta_localterm_inserts() {
    let mut d: HashDataset<Meta<LocalTerm, u32>> = HashDataset::new();
    d.insert(Quad(Meta(blank_term("_:a"), 1), Meta(iri_term("p"), 1), Meta(iri_term("o"), 1), None));
    assert_eq!(d.len(), 1);
}

#[test]
fn round_trip_strip() {
    let s = blank_term("_:a");
    let p = iri_term("p");
    let o = iri_term("o");

    let mq: MetaQuad<u32> = Meta(Quad(Meta(s.clone(), 1), Meta(p.clone(), 1), Meta(o.clone(), 1), None), 1);
    let plain: Quad<LocalTerm> = mq.strip();
    assert_eq!(plain, Quad(s, p, o, None));
}

#[test]
fn meta_quad_full_alias_compiles() {
    let _q: MetaQuadFull<LocalTerm, LocalTerm, LocalTerm, LocalTerm, u8> = Meta(
        Quad(Meta(iri_term("s"), 0), Meta(iri_term("p"), 0), Meta(iri_term("o"), 0), None),
        0,
    );
}
