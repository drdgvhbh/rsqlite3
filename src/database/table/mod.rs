pub use super::{RecordID, Schema, TableValue};
use std::sync::Mutex;

pub trait Pager {
    fn has_free_pages(&self) -> Result<bool, String>;
    fn allocate_page(&mut self) -> Result<(), String>;
    fn insert(&mut self, row: Vec<TableValue>) -> Result<RecordID, String>;
    fn flush(&mut self) -> Result<(), String>;
}

pub struct Table<PA: Pager> {
    schema: Schema,
    pager: Mutex<PA>,
}

impl<PA: Pager> Table<PA> {
    pub fn new(schema: Schema, pager: Mutex<PA>) -> Result<Table<PA>, String> {
        Ok(Table { schema, pager })
    }

    pub fn name(&self) -> &str {
        &self.schema.table_name
    }
}

impl<PA: Pager> super::Table for Table<PA> {
    fn name(&self) -> &str {
        self.name()
    }

    fn insert(&self, row: Vec<TableValue>) -> Result<RecordID, String> {
        let mut pager = self.pager.lock().unwrap();

        if pager.has_free_pages()? {
            pager.allocate_page().map(|_| ())?;
        }

        pager.insert(row)
    }

    fn flush(&self) -> Result<(), String> {
        let mut pager = self.pager.lock().unwrap();
        pager.flush()
    }
}
