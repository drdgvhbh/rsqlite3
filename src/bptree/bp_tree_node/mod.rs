use super::Entry;
use super::{Key, Value};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cmp::Eq;
use std::fmt;
use std::fmt::{Debug, Display};

use std::rc::Rc;

mod internal_node;
mod leaf_node;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum BPTreeNode<K: Key, V: Value> {
    LeafNode(Rc<RefCell<LeafNode<K, V>>>),
    InternalNode(Rc<RefCell<InternalNode<K, V>>>),
}

impl<K: Key + 'static, V: Value + 'static> IntoIterator for BPTreeNode<K, V> {
    type Item = V;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        match &self {
            BPTreeNode::LeafNode(leaf_node) => leaf_node.borrow().clone().into_iter(),
            BPTreeNode::InternalNode(internal_node) => internal_node.borrow().clone().into_iter(),
        }
    }
}

impl<K: Key + 'static, V: Value + 'static> Display for BPTreeNode<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            BPTreeNode::LeafNode(leaf_node) => write!(f, "{}", leaf_node.borrow()),
            BPTreeNode::InternalNode(internal_node) => write!(f, "{}", internal_node.borrow()),
        }
    }
}

impl<K: Key + 'static, V: Value + 'static> BPTreeNode<K, V> {
    pub fn insert(&mut self, entry: Entry<K, V>) -> Result<Option<BPTreeNode<K, V>>, String> {
        match &self {
            BPTreeNode::LeafNode(leaf_node) => leaf_node
                .borrow_mut()
                .insert(entry)
                .map(|opt| opt.map(|rc| BPTreeNode::LeafNode(rc))),
            BPTreeNode::InternalNode(internal_node) => internal_node.borrow_mut().insert(entry),
        }
    }

    fn left_key(&self) -> K {
        match &self {
            BPTreeNode::LeafNode(leaf_node) => leaf_node.borrow().left_key(),
            BPTreeNode::InternalNode(internal_node) => internal_node.borrow().left_key(),
        }
    }

    fn right_key(&self) -> K {
        match &self {
            BPTreeNode::LeafNode(leaf_node) => leaf_node.borrow().right_key(),
            BPTreeNode::InternalNode(internal_node) => internal_node.borrow().right_key(),
        }
    }

    fn capacity(&self) -> usize {
        match &self {
            BPTreeNode::LeafNode(leaf_node) => leaf_node.borrow().entries.capacity(),
            BPTreeNode::InternalNode(internal_node) => internal_node.borrow().entries.capacity(),
        }
    }

    fn len(&self) -> usize {
        match &self {
            BPTreeNode::LeafNode(leaf_node) => leaf_node.borrow().entries.len(),
            BPTreeNode::InternalNode(internal_node) => internal_node.borrow().entries.len(),
        }
    }

    pub fn keys(&self) -> Vec<K> {
        match &self {
            BPTreeNode::LeafNode(leaf_node) => leaf_node.borrow().keys(),
            BPTreeNode::InternalNode(internal_node) => internal_node.borrow().keys(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct LeafNode<K: Key, V: Value> {
    entries: Vec<Entry<K, V>>,
    next: Option<Rc<RefCell<LeafNode<K, V>>>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
struct InternalNodeEntry<K: Key, V: Value> {
    key: K,
    left: BPTreeNode<K, V>,
    right: BPTreeNode<K, V>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct InternalNode<K: Key, V: Value> {
    entries: Vec<InternalNodeEntry<K, V>>,
}
