use super::bp_tree_node::{BPTreeNode, InternalNode, LeafNode};
use super::{Entry, Key, Value};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct BPTree<K: Key, V: Value> {
    pub degree: usize,
    root_node: Option<BPTreeNode<K, V>>,
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
                let new_root = LeafNode::new_from_entry(self.degree, entry);
                self.root_node = Some(BPTreeNode::LeafNode(Rc::new(RefCell::new(new_root))));
            }
            Some(root_node) => match root_node.insert(entry) {
                Err(err) => return Err(err),
                Ok(has_node_split_into_two) => match has_node_split_into_two {
                    None => return Ok(()),
                    Some(split_node) => match (root_node, &split_node) {
                        (BPTreeNode::LeafNode(left), BPTreeNode::LeafNode(right)) => {
                            let new_root =
                                InternalNode::from_two_leaf_nodes(left.clone(), right.clone());
                            self.root_node =
                                Some(BPTreeNode::InternalNode(Rc::new(RefCell::new(new_root))));
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

    use std::fs::File;
    use std::io::prelude::*;

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
}
