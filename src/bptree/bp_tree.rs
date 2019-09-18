use super::bp_tree_node::{BPTreeNode, InternalNode, LeafNode};
use super::{Entry, Key, Value};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct BPTree<K: Key, V: Value> {
    pub degree: usize,
    root_node: Option<BPTreeNode<K, V>>,
}

macro_rules! rcref {
    ($expr:expr) => {{
        Rc::new(RefCell::new($expr))
    }};
}

impl<K: Key + 'static, V: Value + 'static> BPTree<K, V> {
    pub fn new(degree: usize) -> BPTree<K, V> {
        BPTree {
            degree,
            root_node: None,
        }
    }
    pub fn insert(&mut self, entry: Entry<K, V>) -> Result<(), String> {
        match &mut self.root_node {
            None => {
                let new_root = LeafNode::new_from_entry(entry);
                self.root_node = Some(BPTreeNode::LeafNode(rcref!(new_root)));
            }
            Some(root_node) => match root_node.insert(entry, self.degree) {
                Err(err) => return Err(err),
                Ok(has_node_split_into_two) => match has_node_split_into_two {
                    None => return Ok(()),
                    Some(split_node) => match (root_node, &split_node) {
                        (BPTreeNode::LeafNode(left), BPTreeNode::LeafNode(right)) => {
                            self.root_node = Some(BPTreeNode::InternalNode(rcref!(
                                InternalNode::from_leaves(left.clone(), right.clone())
                            )));
                        }
                        (BPTreeNode::InternalNode(left), BPTreeNode::InternalNode(right)) => {
                            self.root_node = Some(BPTreeNode::InternalNode(rcref!(
                                InternalNode::from_internals(left.clone(), right.clone())
                            )));
                        }
                        _ => {
                            debug_assert!(false, "oops");
                        }
                    },
                },
            },
        }

        Ok(())
    }

    /// Returns a depth-first traversal of the keys in the tree.
    ///
    /// Will have duplicates and this function is solely for testing
    /// the construction of the tree.
    #[allow(dead_code)]
    fn keys(&mut self) -> Vec<K> {
        match &self.root_node {
            None => vec![],
            Some(root_node) => root_node.keys(),
        }
    }
}

impl<K: Key + 'static, V: Value + 'static> IntoIterator for BPTree<K, V> {
    type Item = V;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        match self.root_node {
            None => vec![].into_iter(),
            Some(root_node) => root_node.into_iter(),
        }
    }
}

#[cfg(test)]
mod bptree_test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn insertion_works() {
        let mut bptree = BPTree::new(3);
        bptree.insert(Entry::new(1, vec![1, 2, 3])).unwrap();
        bptree.insert(Entry::new(3, vec![400, 500, 600])).unwrap();
        bptree.insert(Entry::new(2, vec![-1, -2, -3])).unwrap();

        assert_eq!(
            bptree.into_iter().collect::<Vec<Vec<i32>>>(),
            vec![vec![1, 2, 3], vec![-1, -2, -3], vec![400, 500, 600]],
        );
    }

    #[test]
    fn tree_is_built_correctly() {
        let mut bptree = BPTree::new(4);
        assert_eq!(bptree.insert(Entry::new(1, vec![1])).is_err(), false);
        assert_eq!(bptree.insert(Entry::new(2, vec![1])).is_err(), false);
        assert_eq!(bptree.insert(Entry::new(3, vec![1])).is_err(), false);
        assert_eq!(bptree.insert(Entry::new(4, vec![1])).is_err(), false);
        assert_eq!(bptree.insert(Entry::new(10, vec![1])).is_err(), false);
        assert_eq!(bptree.insert(Entry::new(11, vec![1])).is_err(), false);
        assert_eq!(bptree.insert(Entry::new(5, vec![1])).is_err(), false);
        assert_eq!(bptree.insert(Entry::new(6, vec![1])).is_err(), false);
        assert_eq!(bptree.insert(Entry::new(20, vec![1])).is_err(), false);
        assert_eq!(bptree.insert(Entry::new(30, vec![1])).is_err(), false);

        println!("{}", bptree.root_node.clone().unwrap());

        assert_eq!(
            bptree.keys(),
            vec![1, 2, 3, 3, 4, 3, 4, 5, 5, 6, 10, 10, 11, 20, 20, 30]
        );
    }
}
