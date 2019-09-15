use super::Entry;
use super::{Key, Value};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cmp::Eq;
use std::fmt::Debug;

use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum BPTreeNode<K: Key, V: Value> {
    LeafNode(Rc<RefCell<LeafNode<K, V>>>),
    InternalNode(Rc<RefCell<InternalNode<K, V>>>),
}

impl<K: Key + 'static, V: Value + 'static> BPTreeNode<K, V> {
    pub fn insert(&mut self, entry: Entry<K, V>) -> Result<Option<BPTreeNode<K, V>>, String> {
        match &self {
            BPTreeNode::LeafNode(leaf_node) => leaf_node
                .borrow_mut()
                .insert(entry)
                .map(|opt| opt.map(|rc| BPTreeNode::LeafNode(rc))),
            BPTreeNode::InternalNode(internal_node) => {
                panic!("oops");
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
struct InternalNodeEntry<K: Key, V: Value> {
    key: K,
    left: BPTreeNode<K, V>,
    right: BPTreeNode<K, V>,
}

impl<K: Key + 'static, V: Value + 'static> InternalNodeEntry<K, V> {
    fn new(key: K, node1: BPTreeNode<K, V>, node2: BPTreeNode<K, V>) {}
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct InternalNode<K: Key, V: Value> {
    entries: Vec<InternalNodeEntry<K, V>>,
}

impl<K: Key + 'static, V: Value + 'static> InternalNode<K, V> {
    pub fn from_two_leaf_nodes(
        left: Rc<RefCell<LeafNode<K, V>>>,
        right: Rc<RefCell<LeafNode<K, V>>>,
    ) -> InternalNode<K, V> {
        debug_assert!(
            right.borrow().entries.len() > 0,
            "right node should have entries"
        );
        let key = right.borrow().entries[0].key.clone();
        let mut entries = Vec::with_capacity(right.borrow().entries.capacity());
        entries.push(InternalNodeEntry {
            key,
            left: BPTreeNode::LeafNode(left),
            right: BPTreeNode::LeafNode(right),
        });
        InternalNode { entries }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct LeafNode<K: Key, V: Value> {
    entries: Vec<Entry<K, V>>,
    next: Option<Rc<RefCell<LeafNode<K, V>>>>,
}

impl<K: Key + 'static, V: Value + 'static> LeafNode<K, V> {
    pub fn new(capacity: usize) -> LeafNode<K, V> {
        LeafNode::new_with_entries(Vec::with_capacity(capacity))
    }

    fn new_with_entries(entries: Vec<Entry<K, V>>) -> LeafNode<K, V> {
        LeafNode {
            entries,
            next: None,
        }
    }

    fn is_full(&self) -> bool {
        self.entries.len() >= self.entries.capacity()
    }

    pub fn insert(
        &mut self,
        entry: Entry<K, V>,
    ) -> Result<Option<Rc<RefCell<LeafNode<K, V>>>>, String> {
        match self.entries.binary_search(&entry) {
            Err(index) => {
                self.entries.insert(index, entry);
                if self.is_full() {
                    return Ok(Some(self.split()));
                }
            }
            Ok(_) => {
                return Err(format!("duplicate entry: {}", entry.key));
            }
        }
        Ok(None)
    }

    fn split(&mut self) -> Rc<RefCell<LeafNode<K, V>>> {
        let mid_index = self.entries.len() / 2;
        let mut new_right = LeafNode::new_with_entries(self.entries.split_off(mid_index));
        new_right.next = self.next.clone();
        self.next = Some(Rc::new(RefCell::new(new_right)));
        self.next.clone().unwrap()
    }
}

#[cfg(test)]
mod leaf_node_test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn insertion_works() {
        let mut leafnode = LeafNode::new(3);
        leafnode.insert(Entry::new(1, vec![1, 2, 3])).unwrap();
        leafnode.insert(Entry::new(3, vec![400, 500, 600])).unwrap();
        let other_half = leafnode
            .insert(Entry::new(2, vec![-1, -2, -3]))
            .unwrap()
            .unwrap();

        assert_eq!(leafnode.entries, vec![Entry::new(1, vec![1, 2, 3]),]);
        assert_eq!(
            other_half.borrow().entries,
            vec![
                Entry::new(2, vec![400, 500, 600]),
                Entry::new(3, vec![-1, -2, -3])
            ]
        );
    }
}
