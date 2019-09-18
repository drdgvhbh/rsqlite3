pub use super::Schema;

pub trait PageAllocator {}

pub struct Table<PA: PageAllocator> {
    schema: Schema,
    page_allocator: PA,
}

impl<PA: PageAllocator> Table<PA> {
    pub fn new(schema: Schema, page_allocator: PA) -> Result<Table<PA>, String> {
        Ok(Table {
            schema,
            page_allocator,
        })
    }
}
