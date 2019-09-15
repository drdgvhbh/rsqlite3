use super::{Key, Value};
use serde;
use serde::ser::Serialize;
use std::cmp::{Eq, Ord, Ordering};
use std::fmt::{Debug, Display};
use std::hash::Hash;

#[derive(serde::Serialize, serde::Deserialize, Eq, Debug, Clone, PartialEq)]
pub struct Entry<K: Key, V: Value> {
    pub key: K,
    pub value: V,
}

impl<K: Key, V: Value> PartialOrd for Entry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Key, V: Value> Ord for Entry<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K: Key, V: Value> Entry<K, V> {
    pub fn new(key: K, value: V) -> Entry<K, V> {
        Entry { key, value }
    }
}
