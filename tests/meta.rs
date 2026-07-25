#![cfg(feature = "meta")]
#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used, clippy::get_unwrap)]

use rdfx::{
    Quad,
    Triple,
    dataset::{BTreeDataset, HashDataset},
    meta::{Meta, Strip},
};

type MetaUsize = Meta<usize, u32>;
type MetaQuadAlias = Meta<Quad<MetaUsize, MetaUsize, MetaUsize, MetaUsize>, u32>;

#[test]
fn strip_meta_quad_collapses_to_plain_quad() {
    let s: usize = 0;
    let p: usize = 1;
    let o: usize = 2;

    let mq: MetaQuadAlias = Meta(Quad(Meta(s, 1), Meta(p, 2), Meta(o, 3), None), 7);

    let stripped: Quad<usize, usize, usize, usize> = mq.strip();
    assert_eq!(stripped, Quad(s, p, o, None));
}

#[test]
fn btree_dataset_of_meta_inserts() {
    let mut d: BTreeDataset<Meta<usize, u32>> = BTreeDataset::new();
    d.insert(Quad(Meta(0usize, 1), Meta(1usize, 1), Meta(2usize, 1), None));
    assert_eq!(d.len(), 1);
}

#[test]
fn meta_eq_distinguishes_metadata() {
    let s: usize = 0;
    let p: usize = 1;
    let o: usize = 2;

    let mut d: BTreeDataset<Meta<usize, u32>> = BTreeDataset::new();
    d.insert(Quad(Meta(s, 1), Meta(p, 1), Meta(o, 1), None));
    d.insert(Quad(Meta(s, 2), Meta(p, 2), Meta(o, 2), None));
    assert_eq!(d.len(), 2);
}

#[test]
fn hash_dataset_of_meta_inserts() {
    let mut d: HashDataset<Meta<usize, u32>> = HashDataset::new();
    d.insert(Quad(Meta(0usize, 1), Meta(1usize, 1), Meta(2usize, 1), None));
    assert_eq!(d.len(), 1);
}

#[test]
fn round_trip_strip_triple() {
    let s: usize = 0;
    let p: usize = 1;
    let o: usize = 2;

    let mt: Triple<Meta<usize, u32>, Meta<usize, u32>, Meta<usize, u32>> = Triple(Meta(s, 1), Meta(p, 1), Meta(o, 1));
    let plain: Triple<usize, usize, usize> = mt.strip();
    assert_eq!(plain, Triple(s, p, o));
}
