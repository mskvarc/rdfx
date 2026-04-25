#![allow(dead_code)]

#[cfg(feature = "ahash")]
pub(crate) type DefaultHashBuilder = ahash::RandomState;
#[cfg(not(feature = "ahash"))]
pub(crate) type DefaultHashBuilder = hashbrown::DefaultHashBuilder;

pub(crate) type HashMap<K, V> = hashbrown::HashMap<K, V, DefaultHashBuilder>;
pub(crate) type HashSet<T> = hashbrown::HashSet<T, DefaultHashBuilder>;
