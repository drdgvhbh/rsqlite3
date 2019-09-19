pub use super::{RecordID, Schema, TableValue};
use std::marker::PhantomData;
use std::sync::Mutex;

pub trait Page {
    fn page_number(&self) -> u32;
    fn first_available_slot_number(&self) -> u8;
    fn write(&self, record: Vec<TableValue>, slot: u8) -> Result<(), String>;
}

pub trait Pager<P: Page> {
    fn first_page_with_available_capacity(&self) -> Result<Option<P>, String>;
    fn allocate_page(&self) -> Result<P, String>;
}

pub struct Table<P: Page, PA: Pager<P>> {
    schema: Schema,
    pager: Mutex<PA>,
    phantom: PhantomData<P>,
}

impl<P: Page, PA: Pager<P>> Table<P, PA> {
    pub fn new(schema: Schema, pager: Mutex<PA>) -> Result<Table<P, PA>, String> {
        Ok(Table {
            schema,
            pager,
            phantom: PhantomData,
        })
    }
}

impl<P: Page, PA: Pager<P>> super::Table for Table<P, PA> {
    fn insert(&self, row: Vec<TableValue>) -> Result<RecordID, String> {
        let pager = self.pager.lock().unwrap();

        if pager.first_page_with_available_capacity()?.is_none() {
            pager.allocate_page().map(|_| ())?;
        }

        let page = pager.first_page_with_available_capacity()?.unwrap();

        let slot_number = page.first_available_slot_number();
        page.write(row, slot_number)?;

        Ok(RecordID::new(page.page_number(), slot_number))
    }
}
