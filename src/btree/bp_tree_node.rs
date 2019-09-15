use super::Entry;
use super::{Key, Value};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cmp::{Eq, Ordering};
use std::fmt;
use std::fmt::{Debug, Display};

use std::rc::Rc;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum BPTreeNode<K: Key, V: Value> {
    LeafNode(Rc<RefCell<LeafNode<K, V>>>),
    InternalNode(Rc<RefCell<InternalNode<K, V>>>),
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
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
struct InternalNodeEntry<K: Key, V: Value> {
    key: K,
    left: BPTreeNode<K, V>,
    right: BPTreeNode<K, V>,
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
    fn new(key: K, left: BPTreeNode<K, V>, right: BPTreeNode<K, V>) -> InternalNodeEntry<K, V> {
        InternalNodeEntry { key, left, right }
    }

    pub fn insert(&mut self, entry: Entry<K, V>) -> Result<Option<BPTreeNode<K, V>>, String> {
        if entry.key < self.key {
            self.left.insert(entry)
        } else {
            self.right.insert(entry)
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
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct InternalNode<K: Key, V: Value> {
    entries: Vec<InternalNodeEntry<K, V>>,
}

impl<K: Key + 'static, V: Value + 'static> Display for InternalNode<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        write!(f, ",InternalNode([")?;
        for item in &self.entries {
            if !first {
                write!(f, ", {}", item)?;
            } else {
                write!(f, "{}", item)?;
            }
            first = false;
        }
        write!(f, "])")?;
        Ok(())
    }
}

impl<K: Key + 'static, V: Value + 'static> InternalNode<K, V> {
    pub fn from_two_leaf_nodes(
        left: Rc<RefCell<LeafNode<K, V>>>,
        right: Rc<RefCell<LeafNode<K, V>>>,
    ) -> InternalNode<K, V> {
        InternalNode::from_two_nodes(BPTreeNode::LeafNode(left), BPTreeNode::LeafNode(right))
    }

    fn from_two_nodes(left: BPTreeNode<K, V>, right: BPTreeNode<K, V>) -> InternalNode<K, V> {
        debug_assert!(right.capacity() > 0, "right node should have entries");
        let key = right.left_key();
        let mut entries = Vec::with_capacity(right.capacity());
        entries.push(InternalNodeEntry::new(key, left, right));
        InternalNode { entries }
    }

    fn is_full(&self) -> bool {
        self.entries.len() >= self.entries.capacity()
    }

    fn left_key(&self) -> K {
        let entries = &self.entries;
        debug_assert!(
            entries.len() > 0,
            "internal node should have at least 1 entry"
        );
        return entries[0].key.clone();
    }

    fn right_key(&self) -> K {
        let entries = &self.entries;
        debug_assert!(
            entries.len() > 0,
            "internal node should have at least 1 entry"
        );
        return entries[entries.len() - 1].key.clone();
    }

    pub fn insert(&mut self, entry: Entry<K, V>) -> Result<Option<BPTreeNode<K, V>>, String> {
        match self
            .entries
            .binary_search_by_key(&entry.key, |internal_node| internal_node.key.clone())
        {
            Err(index) => {
                let mut existing_index = index;
                if existing_index == self.entries.len() {
                    existing_index -= 1;
                }

                let key = entry.key.clone();
                match self.entries[existing_index].insert(entry) {
                    Err(err) => return Err(err),
                    Ok(has_node_split_into_two) => match has_node_split_into_two {
                        None => {}
                        Some(split_node) => {
                            let new_internal_node_entry = InternalNodeEntry::new(
                                split_node.left_key(),
                                self.entries[existing_index].side(&key),
                                split_node,
                            );
                            self.insert_node_at(new_internal_node_entry, index);
                        }
                    },
                }
                /*                 if self.is_full() {
                    return Ok(Some(self.split()));
                } */
            }
            Ok(_) => {
                return Err(format!("duplicate entry: {}", entry.key));
            }
        }
        Ok(None)
    }

    fn insert_node_at(&mut self, entry: InternalNodeEntry<K, V>, index: usize) {
        self.entries.insert(index, entry);
        // self.entries.insert(index, InternalNode { entries: Vec::with_capacity(self.entries.capacity())})
        // let left = self.entries.get(index);
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct LeafNode<K: Key, V: Value> {
    entries: Vec<Entry<K, V>>,
    next: Option<Rc<RefCell<LeafNode<K, V>>>>,
}

impl<K: Key + 'static, V: Value + 'static> Display for LeafNode<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LeafNode({:#?})",
            self.entries
                .iter()
                .map(|entry| entry.key.clone())
                .collect::<Vec<K>>()
        )
    }
}

impl<K: Key + 'static, V: Value + 'static> IntoIterator for LeafNode<K, V> {
    type Item = V;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        let mut all_entries = self
            .entries
            .iter()
            .map(|entry| entry.value.clone())
            .collect::<Vec<V>>();
        match &self.next {
            None => {}
            Some(next_entries) => {
                all_entries.extend(next_entries.borrow().clone().into_iter());
            }
        }
        all_entries.into_iter()
    }
}

impl<K: Key + 'static, V: Value + 'static> LeafNode<K, V> {
    pub fn new(capacity: usize) -> LeafNode<K, V> {
        LeafNode::new_with_entries(Vec::with_capacity(capacity))
    }

    pub fn new_from_entry(capacity: usize, entry: Entry<K, V>) -> LeafNode<K, V> {
        let mut entries = Vec::with_capacity(capacity);
        entries.push(entry);
        LeafNode::new_with_entries(entries)
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

    fn left_key(&self) -> K {
        let entries = &self.entries;
        debug_assert!(entries.len() > 0, "leaf node should have at least 1 entry");
        return entries[0].key.clone();
    }

    fn right_key(&self) -> K {
        let entries = &self.entries;
        debug_assert!(
            entries.len() > 0,
            "internal node should have at least 1 entry"
        );
        return entries[entries.len() - 1].key.clone();
    }
}

#[cfg(test)]
mod leaf_node_test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn has_correct_iteration_order_after_insertion() {
        let capacity = 3;

        let mut leafnode = LeafNode::new(capacity);
        assert_eq!(
            leafnode.insert(Entry::new(1, vec![1, 2, 3])).is_err(),
            false
        );
        assert_eq!(
            leafnode.insert(Entry::new(3, vec![400, 500, 600])).is_err(),
            false
        );
        assert_eq!(
            leafnode.insert(Entry::new(2, vec![-1, -2, -3])).is_err(),
            false
        );

        assert_eq!(
            leafnode.into_iter().collect::<Vec<Vec<i32>>>(),
            vec![vec![1, 2, 3], vec![-1, -2, -3], vec![400, 500, 600]]
        );
    }

    #[test]
    fn nodes_are_split_when_capacity_is_reached() {
        let capacity = 3;

        let mut leafnode = LeafNode::new(capacity);
        assert_eq!(
            leafnode.insert(Entry::new(1, vec![1, 2, 3])).is_err(),
            false
        );
        assert_eq!(
            leafnode.insert(Entry::new(3, vec![400, 500, 600])).is_err(),
            false
        );
        assert_eq!(
            leafnode.insert(Entry::new(2, vec![-1, -2, -3])).is_err(),
            false
        );

        assert_eq!(leafnode.entries, vec![Entry::new(1, vec![1, 2, 3]),]);
        assert_ne!(leafnode.next, None);
        assert_eq!(
            leafnode.next.unwrap().borrow().entries,
            vec![
                Entry::new(2, vec![-1, -2, -3]),
                Entry::new(3, vec![400, 500, 600]),
            ]
        );
    }

    #[test]
    fn duplicate_insertion_fails() {
        let capacity = 3;

        let mut leafnode = LeafNode::new(capacity);
        assert_eq!(
            leafnode.insert(Entry::new(1, vec![1, 2, 3])).is_err(),
            false
        );
        assert_eq!(
            leafnode.insert(Entry::new(3, vec![400, 500, 600])).is_err(),
            false
        );
        assert_eq!(
            leafnode.insert(Entry::new(3, vec![-1, -2, -3])).is_err(),
            true
        );
    }
}
