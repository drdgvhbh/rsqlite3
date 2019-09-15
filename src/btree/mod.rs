use serde::ser::Serialize;
use std::cell::RefCell;
use std::cmp::{Eq, Ord, Ordering};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::rc::{Rc, Weak};

pub struct BPTree {}

impl BPTree {}

trait Key = Hash + Serialize + Eq + Ord + Display + Debug;
trait Value = Serialize + Eq + Debug;

#[derive(Eq, Debug, Clone)]
struct Entry<K: Key, V: Value> {
    key: K,
    value: V,
}

impl<K: Key, V: Value> PartialEq for Entry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
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

#[derive(Debug, Clone)]
struct BTreeNode<K: Key, V: Value> {
    entries: Vec<Entry<K, V>>,
    // right: Option<Weak<RefCell<BTreeNode<K, V>>>>,
    right: Option<Rc<BTreeNode<K, V>>>,
    next: Option<Rc<RefCell<BTreeNode<K, V>>>>,
}

trait InternalNode<K: Key, V: Value> {
    fn insert(&mut self, entry: Entry<K, V>)
        -> Result<Option<Box<dyn InternalNode<K, V>>>, String>;
}

trait LeafNode<K: Key, V: Value>: Debug {
    fn insert(&mut self, entry: Entry<K, V>) -> Result<Option<Rc<dyn LeafNode<K, V>>>, String>;
}

impl<K: Key, V: Value> BTreeNode<K, V> {
    pub fn new(capacity: usize) -> BTreeNode<K, V> {
        BTreeNode::new_with_entries(Vec::with_capacity(capacity))
    }

    fn new_with_entries(entries: Vec<Entry<K, V>>) -> BTreeNode<K, V> {
        BTreeNode {
            entries,
            right: None,
            next: None,
        }
    }

    fn is_full(&self) -> bool {
        self.entries.len() >= self.entries.capacity()
    }

    /*     pub fn insert(&mut self, entry: Entry<K, V>) -> Result<(), String> {
        match self.entries.binary_search(&entry) {
            Err(index) => {
                self.entries.insert(index, entry);
            }
            Ok(_) => {
                return Err(format!("duplicate entry: {}", entry.key));
            }
        }
        Ok(())
    } */

    fn split(&mut self) -> Rc<BTreeNode<K, V>> {
        let mid_index = self.entries.len() / 2;
        let mut new_right = BTreeNode::new_with_entries(self.entries.split_off(mid_index));
        new_right.right = self.right.clone();
        self.right = Some(Rc::new(new_right));
        self.right.clone().unwrap()
    }
}

impl<K: Key + 'static, V: Value + 'static> LeafNode<K, V> for BTreeNode<K, V> {
    fn insert(&mut self, entry: Entry<K, V>) -> Result<Option<Rc<dyn LeafNode<K, V>>>, String> {
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
}

#[cfg(test)]
mod leaf_node_test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn insertion_works() {
        let mut leafnode = BTreeNode::new(3);
        leafnode.insert(Entry::new(1, vec![1, 2, 3])).unwrap();
        leafnode.insert(Entry::new(3, vec![400, 500, 600])).unwrap();
        let other_half = leafnode.insert(Entry::new(2, vec![-1, -2, -3])).unwrap();

        println!("{:#?} {:#?}", leafnode, other_half);
        assert_eq!(
            leafnode.entries,
            vec![
                Entry::new(1, vec![1, 2, 3]),
                Entry::new(2, vec![400, 500, 600]),
                Entry::new(3, vec![-1, -2, -3])
            ]
        )
    }
}
