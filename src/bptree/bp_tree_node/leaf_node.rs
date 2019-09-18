use super::Entry;
use super::LeafNode;
use super::{Key, Value};
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use super::super::Serializer;

macro_rules! rcref {
    ($expr:expr) => {{
        Rc::new(RefCell::new($expr))
    }};
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
    pub fn new() -> LeafNode<K, V> {
        LeafNode::new_with_entries(vec![])
    }

    pub fn new_from_entry(entry: Entry<K, V>) -> LeafNode<K, V> {
        LeafNode::new_with_entries(vec![entry])
    }

    fn new_with_entries(entries: Vec<Entry<K, V>>) -> LeafNode<K, V> {
        LeafNode {
            entries,
            next: None,
        }
    }

    pub fn insert(
        &mut self,
        entry: Entry<K, V>,
        page_byte_size: usize,
        serializer: Serializer,
    ) -> Result<Option<Rc<RefCell<LeafNode<K, V>>>>, String> {
        match self.entries.binary_search(&entry) {
            Err(index) => {
                self.entries.insert(index, entry);
                if serializer.serialize(&self.entries).len() >= page_byte_size {
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
        let right_split = self.entries.split_off(mid_index);
        let mut new_right = LeafNode::new_with_entries(right_split);
        new_right.next = self.next.clone();
        self.next = Some(rcref!(new_right));
        self.next.clone().unwrap()
    }

    pub fn left_key(&self) -> K {
        let entries = &self.entries;
        debug_assert!(entries.len() > 0, "leaf node should have at least 1 entry");
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

    pub fn keys(&self) -> Vec<K> {
        self.entries
            .iter()
            .map(|entry| entry.key.clone())
            .collect::<Vec<K>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! new_leaf_node {
        ($page_byte_size:expr, $($key:expr => $value:expr),*) => {{
            let mut leafnode = LeafNode::<i32, Vec<i32>>::new();
            $(assert_eq!(leafnode.insert(Entry::new($key, $value), $page_byte_size, Serializer::Mock).is_err(), false);)*
            leafnode
        }};
    }

    #[test]
    fn has_correct_iteration_order_after_insertion() {
        let page_byte_size = 3;

        let leafnode = new_leaf_node!(
            page_byte_size, 
            1 => vec![1,2,3], 
            3 => vec![400, 500, 600], 
            2 => vec![-1, -2, -3]);

        assert_eq!(
            leafnode.into_iter().collect::<Vec<Vec<i32>>>(),
            vec![vec![1, 2, 3], vec![-1, -2, -3], vec![400, 500, 600]]
        );
    }

    #[test]
    fn nodes_are_split_when_page_byte_size_is_reached() {
        let page_byte_size = 3;

        let leafnode = new_leaf_node!(
            page_byte_size, 
            1 => vec![1,2,3], 
            3 => vec![400, 500, 600], 
            2 => vec![-1, -2, -3]);
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
    fn nodes_are_split_when_page_byte_size_is_reached2() {
        let page_byte_size = 4;

        let leafnode = new_leaf_node!(
            page_byte_size,
            1 => vec![1,2,3],
            3 => vec![400, 500, 600],
            2 => vec![-1, -2, -3],
            4 => vec![-1, -2, -3]
        );
        assert_eq!(
            leafnode.entries,
            vec![
                Entry::new(1, vec![1, 2, 3]),
                Entry::new(2, vec![-1, -2, -3])
            ]
        );
        assert_ne!(leafnode.next, None);
        assert_eq!(
            leafnode.next.unwrap().borrow().entries,
            vec![
                Entry::new(3, vec![400, 500, 600]),
                Entry::new(4, vec![-1, -2, -3]),
            ]
        );
    }

    #[test]
    fn duplicate_insertion_fails() {
        let page_byte_size = 3;
        let mut leafnode = new_leaf_node!(
            page_byte_size,  
            1 => vec![1,2,3], 
            3 => vec![400, 500, 600]);
        assert_eq!(
            leafnode.insert(Entry::new(3, vec![-1, -2, -3]), page_byte_size, Serializer::Mock).is_err(),
            true
        );
    }
}
