use super::BPTree;
use crate::{ast::Value, bptree};

impl BPTree for bptree::BPTree<Value, Vec<Value>> {
    fn insert(&mut self, key: Value, value: Vec<Value>) -> Result<(), String> {
        self.insert(bptree::Entry::new(key, value))
    }
}
