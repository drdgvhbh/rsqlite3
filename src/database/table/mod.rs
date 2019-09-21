pub use super::{RecordID, Schema, TableValue};

pub trait Pager {
    fn has_free_pages(&self) -> Result<bool, String>;
    fn allocate_page(&mut self) -> Result<(), String>;
    fn insert(&mut self, row: Vec<TableValue>) -> Result<RecordID, String>;
    fn flush(&mut self) -> Result<(), String>;
    fn rows(&self) -> Result<Vec<Vec<&TableValue>>, String>;
}

pub struct Table<PA: Pager> {
    schema: Schema,
    pager: PA,
}

impl<PA: Pager> Table<PA> {
    pub fn new(schema: Schema, pager: PA) -> Result<Table<PA>, String> {
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

    fn insert(&mut self, row: Vec<TableValue>) -> Result<RecordID, String> {
        if self.pager.has_free_pages()? {
            self.pager.allocate_page().map(|_| ())?;
        }

        self.pager.insert(row)
    }

    fn flush(&mut self) -> Result<(), String> {
        self.pager.flush()
    }

    fn rows(&self) -> Result<Vec<Vec<&TableValue>>, String> {
        self.pager.rows()
    }
}
