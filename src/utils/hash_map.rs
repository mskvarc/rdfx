#![allow(dead_code)]

use std::hash::Hash;

#[cfg(feature = "ahash")]
pub(crate) type DefaultHashBuilder = ahash::RandomState;
#[cfg(not(feature = "ahash"))]
pub(crate) type DefaultHashBuilder = hashbrown::DefaultHashBuilder;

pub(crate) type HashMap<K, V> = hashbrown::HashMap<K, V, DefaultHashBuilder>;
pub(crate) type HashSet<T> = hashbrown::HashSet<T, DefaultHashBuilder>;

/// Hashes `value` with the configured backend hasher.
///
/// Bottles the `BuildHasher` trait usage so callers don't have to import it.
/// Always dispatches through the trait method to keep results identical
/// across `ahash`/`foldhash` backends — `ahash` also exposes an inherent
/// `hash_one`, which would shadow the trait method at the call site and
/// trigger "unused import" warnings if callers tried to bring `BuildHasher`
/// into scope themselves.
#[inline]
pub(crate) fn hash_one<T: Hash>(builder: &DefaultHashBuilder, value: T) -> u64 {
    <DefaultHashBuilder as std::hash::BuildHasher>::hash_one(builder, value)
}
