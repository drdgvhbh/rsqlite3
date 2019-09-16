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
    pub fn from_two_leaf_nodes(
        left: Rc<RefCell<LeafNode<K, V>>>,
        right: Rc<RefCell<LeafNode<K, V>>>,
    ) -> InternalNode<K, V> {
        InternalNode::from_two_nodes(BPTreeNode::LeafNode(left), BPTreeNode::LeafNode(right))
    }

    pub fn from_two_internal_nodes(
        left: Rc<RefCell<InternalNode<K, V>>>,
        right: Rc<RefCell<InternalNode<K, V>>>,
    ) -> InternalNode<K, V> {
        debug_assert!(
            right.borrow().entries.len() > 0,
            "right node should have entries"
        );
        let key = right.borrow().left_key();
        let mut entries = Vec::with_capacity(right.borrow().entries.capacity());
        let new_right = Rc::new(RefCell::new(InternalNode::new_with_entries(
            right.borrow().entries.clone()[1..].to_vec(),
        )));
        entries.push(InternalNodeEntry::new(
            key,
            BPTreeNode::InternalNode(left),
            BPTreeNode::InternalNode(new_right),
        ));
        InternalNode { entries }
    }

    fn from_two_nodes(left: BPTreeNode<K, V>, right: BPTreeNode<K, V>) -> InternalNode<K, V> {
        debug_assert!(right.len() > 0, "right node should have entries");
        let key = right.left_key();
        let mut entries = Vec::with_capacity(right.capacity());
        entries.push(InternalNodeEntry::new(key, left, right));
        InternalNode { entries }
    }

    fn new_with_entries(entries: Vec<InternalNodeEntry<K, V>>) -> InternalNode<K, V> {
        InternalNode { entries }
    }

    fn is_full(&self) -> bool {
        self.entries.len() >= self.entries.capacity()
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
                if self.is_full() {
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
        let mut split_with_correct_alloc = Vec::with_capacity(self.entries.capacity());
        split_with_correct_alloc.extend(right_split);
        let new_right = InternalNode::new_with_entries(split_with_correct_alloc);
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

    fn create_internal_node() -> InternalNode<i32, Vec<i32>> {
        let capacity = 3;
        let mut left_leafnode = LeafNode::new(capacity);
        let mut right_leafnode = LeafNode::new(capacity);
        left_leafnode.insert(Entry::new(1, vec![1, 2, 3])).unwrap();
        right_leafnode
            .insert(Entry::new(3, vec![400, 500, 600]))
            .unwrap();
        right_leafnode
            .insert(Entry::new(2, vec![-1, -2, -3]))
            .unwrap();
        let rc_right_node = Some(Rc::new(RefCell::new(right_leafnode)));
        left_leafnode.next = rc_right_node.clone();

        InternalNode::from_two_leaf_nodes(
            Rc::new(RefCell::new(left_leafnode)),
            rc_right_node.unwrap().clone(),
        )
    }

    #[test]
    fn has_correct_iteration_order_after_insertion() {
        assert_eq!(
            create_internal_node()
                .into_iter()
                .collect::<Vec<Vec<i32>>>(),
            vec![vec![1, 2, 3], vec![-1, -2, -3], vec![400, 500, 600]]
        );
    }

    #[test]
    fn internal_node_is_built_correctly() {
        let mut inode = create_internal_node();
        assert_eq!(inode.insert(Entry::new(4, vec![1])).is_err(), false);

        assert_eq!(inode.keys(), vec![1, 2, 2, 2, 3, 3, 4]);
    }

    #[test]
    fn internal_node_is_built_correctly2() {
        let capacity = 4;
        let mut left_leafnode = LeafNode::new(capacity);
        left_leafnode.insert(Entry::new(1, vec![1, 2, 3])).unwrap();
        left_leafnode.insert(Entry::new(2, vec![1, 2, 3])).unwrap();
        left_leafnode.insert(Entry::new(3, vec![1, 2, 3])).unwrap();
        let right_leafnode = left_leafnode
            .insert(Entry::new(4, vec![1, 2, 3]))
            .unwrap()
            .unwrap();

        let rc_right_node = Some(right_leafnode);
        left_leafnode.next = rc_right_node.clone();

        let inode = InternalNode::from_two_leaf_nodes(
            Rc::new(RefCell::new(left_leafnode)),
            rc_right_node.unwrap().clone(),
        );

        assert_eq!(inode.keys(), vec![1, 2, 3, 3, 4]);
    }

    #[test]
    fn internal_node_is_built_correctly3() {
        let capacity = 4;
        let mut left_leafnode = LeafNode::new(capacity);
        left_leafnode.insert(Entry::new(1, vec![1, 2, 3])).unwrap();
        left_leafnode.insert(Entry::new(2, vec![1, 2, 3])).unwrap();
        left_leafnode.insert(Entry::new(3, vec![1, 2, 3])).unwrap();
        let right_leafnode = left_leafnode
            .insert(Entry::new(4, vec![1, 2, 3]))
            .unwrap()
            .unwrap();

        let rc_right_node = Some(right_leafnode);
        left_leafnode.next = rc_right_node.clone();

        let mut inode = InternalNode::from_two_leaf_nodes(
            Rc::new(RefCell::new(left_leafnode)),
            rc_right_node.unwrap().clone(),
        );
        assert_eq!(inode.insert(Entry::new(10, vec![1])).is_err(), false);
        assert_eq!(inode.insert(Entry::new(11, vec![1])).is_err(), false);
        assert_eq!(inode.insert(Entry::new(5, vec![1])).is_err(), false);
        assert_eq!(inode.insert(Entry::new(6, vec![1])).is_err(), false);
        assert_eq!(inode.insert(Entry::new(20, vec![1])).is_err(), false);

        println!("{}", inode);

        assert_eq!(
            inode.keys(),
            vec![1, 2, 3, 3, 4, 3, 4, 5, 5, 6, 5, 6, 10, 10, 11, 20]
        );
    }
}
