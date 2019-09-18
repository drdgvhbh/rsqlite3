use super::Entry;
use super::{BPTreeNode, InternalNode, InternalNodeEntry, LeafNode};
use super::{Key, Value};
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

mod internal_node_entry;

impl<K: Key + 'static, V: Value + 'static> IntoIterator for InternalNode<K, V> {
    type Item = V;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        debug_assert!(!self.entries.is_empty(), "internal node must have entries");
        self.entries[0].clone().into_iter()
    }
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
    pub fn from_leaves(
        left: Rc<RefCell<LeafNode<K, V>>>,
        right: Rc<RefCell<LeafNode<K, V>>>,
    ) -> InternalNode<K, V> {
        InternalNode::from_two_nodes(BPTreeNode::LeafNode(left), BPTreeNode::LeafNode(right))
    }

    pub fn from_internals(
        left: Rc<RefCell<InternalNode<K, V>>>,
        right: Rc<RefCell<InternalNode<K, V>>>,
    ) -> InternalNode<K, V> {
        debug_assert!(
            right.borrow().entries.len() > 0,
            "right node should have entries"
        );
        let key = right.borrow().left_key();
        let new_right = Rc::new(RefCell::new(InternalNode::new_with_entries(
            right.borrow().entries.clone()[1..].to_vec(),
        )));
        InternalNode { entries: vec![InternalNodeEntry::new(
            key,
            BPTreeNode::InternalNode(left),
            BPTreeNode::InternalNode(new_right),
        )] }
    }

    fn from_two_nodes(left: BPTreeNode<K, V>, right: BPTreeNode<K, V>) -> InternalNode<K, V> {
        debug_assert!(right.len() > 0, "right node should have entries");
        let key = right.left_key();
        InternalNode { entries: vec![InternalNodeEntry::new(key, left, right)] }
    }

    fn new_with_entries(entries: Vec<InternalNodeEntry<K, V>>) -> InternalNode<K, V> {
        InternalNode { entries }
    }

    pub fn left_key(&self) -> K {
        let entries = &self.entries;
        debug_assert!(
            entries.len() > 0,
            "internal node should have at least 1 entry"
        );
        return entries[0].key.clone();
    }

    pub fn right_key(&self) -> K {
        let entries = &self.entries;
        debug_assert!(
            entries.len() > 0,
            "internal node should have at least 1 entry"
        );
        return entries[entries.len() - 1].key.clone();
    }

    pub fn insert(&mut self, entry: Entry<K, V>, degree: usize) -> Result<Option<BPTreeNode<K, V>>, String> {
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
                match self.entries[existing_index].insert(entry, degree) {
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
                if self.entries.len() >= degree {
                    return Ok(Some(BPTreeNode::InternalNode(self.split())));
                }
            } 
            Ok(_) => {
                return Err(format!("duplicate entry: {}", entry.key));
            }
        }
        Ok(None)
    }

    fn split(&mut self) -> Rc<RefCell<InternalNode<K, V>>> {
        let mid_index = self.entries.len() / 2;
        let right_split = self.entries.split_off(mid_index);
        let new_right = InternalNode::new_with_entries(right_split);
        Rc::new(RefCell::new(new_right))
    }

    fn insert_node_at(&mut self, entry: InternalNodeEntry<K, V>, index: usize) {
        let entry_clone = entry.clone();
        self.entries.insert(index, entry);

        match self.entries.get_mut(index - 1) {
            None => {}
            Some(left) => {
                left.right = entry_clone.left.clone();
            }
        }
        match self.entries.get_mut(index + 1) {
            None => {}
            Some(right) => {
                right.left = entry_clone.right.clone();
            }
        }
    }

    pub fn keys(&self) -> Vec<K> {
        let mut keys = vec![];
        for entry in &self.entries {
            keys.extend(entry.keys());
        }

        keys
    }
}

#[cfg(test)]
mod internal_node_test {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! new_leaf_node {
        ($degree:expr, $($key:expr => $value:expr),*) => {{
            let mut leafnode = LeafNode::<i32, Vec<i32>>::new();
            $(assert_eq!(leafnode.insert(Entry::new($key, $value), $degree).is_err(), false);)*
            leafnode
        }};
    }

    macro_rules! new_internal_node {
        ($left:expr, $right:expr) => {{
            let rc_right_node = Some(Rc::new(RefCell::new($right)));
            $left.next = rc_right_node.clone();

            InternalNode::from_leaves(
                Rc::new(RefCell::new($left)),
                rc_right_node.unwrap().clone(),
            )
        }};
    }

    macro_rules! insert {
        ($inode:expr, $degree:expr, $($key:expr => $value:expr),*) => {{
            $(assert_eq!($inode.insert(Entry::new($key, $value), $degree).is_err(), false);)*
        }};
    }

    #[test]
    fn has_correct_iteration_order_after_insertion() {
        let degree = 3;

        let mut left_node = new_leaf_node!(degree, 1 => vec![1, 2, 3]);
        let internal_node = new_internal_node!(
            left_node,
            new_leaf_node!(
                degree, 
                3 => vec![400, 500, 600], 
                2 => vec![-1, -2, -3])
        );
        assert_eq!(
            internal_node.into_iter().collect::<Vec<Vec<i32>>>(),
            vec![vec![1, 2, 3], vec![-1, -2, -3], vec![400, 500, 600]]
        );
    }

    #[test]
    fn internal_node_is_built_correctly() {
        let degree = 3;

        let mut left_node = new_leaf_node!(degree, 1 => vec![1,2,3]);
        let mut internal_node = new_internal_node!(
            left_node,
            new_leaf_node!(
                degree, 
                3 => vec![400, 500, 600], 
                2 => vec![-1, -2, -3])
        );
        insert!(internal_node, degree, 4 => vec![1]);
        assert_eq!(internal_node.keys(), vec![1, 2, 2, 2, 3, 3, 4]);
    }

    #[test]
    fn internal_node_is_built_correctly2() {
        let degree = 4;

        let mut left_leafnode = new_leaf_node!(
            degree, 
            1 => vec![1, 2, 3],
            2 => vec![1, 2, 3],
            3 => vec![1, 2, 3]);
        let right_leafnode = left_leafnode
            .insert(Entry::new(4, vec![1, 2, 3]), degree)
            .unwrap()
            .unwrap();

        let rc_right_node = Some(right_leafnode);
        left_leafnode.next = rc_right_node.clone();

        let internal_node = InternalNode::from_leaves(
            Rc::new(RefCell::new(left_leafnode)),
            rc_right_node.unwrap().clone(),
        );

        assert_eq!(internal_node.keys(), vec![1, 2, 3, 3, 4]);
    }

    #[test]
    fn internal_node_is_built_correctly3() {
        let degree = 4;

        let mut left_leafnode = new_leaf_node!(
            degree, 
            1 => vec![1, 2, 3],
            2 => vec![1, 2, 3],
            3 => vec![1, 2, 3]);
        let right_leafnode = left_leafnode
            .insert(Entry::new(4, vec![1, 2, 3]), degree)
            .unwrap()
            .unwrap();

        let rc_right_node = Some(right_leafnode);
        left_leafnode.next = rc_right_node.clone();

        let mut internal_node = InternalNode::from_leaves(
            Rc::new(RefCell::new(left_leafnode)),
            rc_right_node.unwrap().clone(),
        );

        insert!(internal_node,
            degree,
            10 => vec![1],
            11 => vec![1],
            5 => vec![1],
            6 => vec![1],
            20 => vec![1]
        );
        assert_eq!(
            internal_node.keys(),
            vec![1, 2, 3, 3, 4, 3, 4, 5, 5, 6, 5, 6, 10, 10, 11, 20]
        );
    }
}
