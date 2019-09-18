use super::Entry;
use super::{BPTreeNode, InternalNodeEntry};
use super::{Key, Value};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Display;

impl<K: Key + 'static, V: Value + 'static> IntoIterator for InternalNodeEntry<K, V> {
    type Item = V;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.left.into_iter()
    }
}

impl<K: Key, V: Value> PartialOrd for InternalNodeEntry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Key, V: Value> Ord for InternalNodeEntry<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K: Key + 'static, V: Value + 'static> Display for InternalNodeEntry<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InternalNodeEntry(key: {}, left: {}, right: {})",
            self.key, self.left, self.right,
        )
    }
}

impl<K: Key + 'static, V: Value + 'static> InternalNodeEntry<K, V> {
    pub fn new(key: K, left: BPTreeNode<K, V>, right: BPTreeNode<K, V>) -> InternalNodeEntry<K, V> {
        InternalNodeEntry { key, left, right }
    }

    pub fn insert(
        &mut self,
        entry: Entry<K, V>,
        degree: usize,
    ) -> Result<Option<BPTreeNode<K, V>>, String> {
        if entry.key < self.key {
            self.left.insert(entry, degree)
        } else {
            self.right.insert(entry, degree)
        }
    }

    pub fn side(&self, key: &K) -> BPTreeNode<K, V> {
        if key < &self.key {
            match &self.left {
                BPTreeNode::LeafNode(leaf_node) => BPTreeNode::LeafNode(leaf_node.clone()),
                BPTreeNode::InternalNode(internal_node) => {
                    BPTreeNode::InternalNode(internal_node.clone())
                }
            }
        } else {
            match &self.right {
                BPTreeNode::LeafNode(leaf_node) => BPTreeNode::LeafNode(leaf_node.clone()),
                BPTreeNode::InternalNode(internal_node) => {
                    BPTreeNode::InternalNode(internal_node.clone())
                }
            }
        }
    }

    pub fn keys(&self) -> Vec<K> {
        let mut keys = self.left.keys();
        keys.push(self.key.clone());
        keys.extend(self.right.keys());

        keys
    }
}
