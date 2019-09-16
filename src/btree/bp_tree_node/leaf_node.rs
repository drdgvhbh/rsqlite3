use super::Entry;
use super::LeafNode;
use super::{Key, Value};
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

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
        let right_split = self.entries.split_off(mid_index);
        let mut split_with_correct_alloc = Vec::with_capacity(self.entries.capacity());
        split_with_correct_alloc.extend(right_split);
        let mut new_right = LeafNode::new_with_entries(split_with_correct_alloc);
        new_right.next = self.next.clone();
        self.next = Some(Rc::new(RefCell::new(new_right)));
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

    fn create_leaf_node() -> LeafNode<i32, Vec<i32>> {
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

        leafnode
    }

    fn create_leaf_node_cap4() -> LeafNode<i32, Vec<i32>> {
        let capacity = 4;

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
            leafnode.insert(Entry::new(4, vec![-1, -2, -3])).is_err(),
            false
        );

        leafnode
    }

    #[test]
    fn has_correct_iteration_order_after_insertion() {
        assert_eq!(
            create_leaf_node().into_iter().collect::<Vec<Vec<i32>>>(),
            vec![vec![1, 2, 3], vec![-1, -2, -3], vec![400, 500, 600]]
        );
    }

    #[test]
    fn nodes_are_split_when_capacity_is_reached() {
        let leafnode = create_leaf_node();
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
    fn nodes_are_split_when_capacity_is_reached2() {
        let leafnode = create_leaf_node_cap4();
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
