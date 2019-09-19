use super::{data::TableSerializationSize, table, Schema, Serializer, TableValue};
use positioned_io_preview::RandomAccessFile;
use positioned_io_preview::WriteAt;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
pub struct Page {
    page_number: u32,
}

impl table::Page for Page {
    fn page_number(&self) -> u32 {
        self.page_number
    }

    fn first_available_slot_number(&self) -> u8 {
        panic!("TODO")
    }

    fn write(&self, record: Vec<TableValue>, slot: u8) -> Result<(), String> {
        panic!("TODO")
    }
}

pub struct Pager<S: Serializer> {
    file: RandomAccessFile,
    page_byte_size: usize,
    serializer: S,
}

impl<S: Serializer> table::Pager<Page> for Pager<S> {
    fn first_page_with_available_capacity(&self) -> Result<Option<Page>, String> {
        panic!("TODO")
    }
    fn allocate_page(&self) -> Result<Page, String> {
        panic!("TODO")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageHeader {
    schema: Schema,
    num_pages: usize,
    page_byte_size: usize,
    slot_capacity: usize,
}

impl<S: Serializer> Pager<S> {
    pub fn new(file: RandomAccessFile, page_byte_size: usize, serializer: S) -> Pager<S> {
        Pager {
            file,
            page_byte_size,
            serializer,
        }
    }

    pub fn write_header(&mut self, schema: Schema) -> Result<(), String> {
        let tss = schema.size(&self.serializer);
        let mut slot_capacity = self.page_byte_size / tss.row_size;
        let mut slot_capacity_bytes = self.serializer.serialize(&slot_capacity);
        let mut slots_bytes = self.serializer.serialize(&vec![false; slot_capacity]);

        while slot_capacity_bytes.len()
            + slots_bytes.len()
            + tss.vector_size
            + tss.row_size * slot_capacity
            > self.page_byte_size
        {
            slot_capacity -= 1;
            slot_capacity_bytes = self.serializer.serialize(&slot_capacity);
            slots_bytes = self.serializer.serialize(&vec![false; slot_capacity]);
        }

        let page_header = PageHeader {
            schema,
            num_pages: 0,
            page_byte_size: self.page_byte_size,
            slot_capacity,
        };
        let page_header_bytes = self.serializer.serialize(&page_header);
        if page_header_bytes.len() > self.page_byte_size {
            return Err("page size is not large enough to fight page header".to_string());
        }

        write(&mut self.file, 0, &page_header_bytes, &self.serializer)?;

        Ok(())
    }
}

fn write<S: Serializer>(
    file: &mut RandomAccessFile,
    position: u64,
    bytes: &[u8],
    serializer: &S,
) -> Result<usize, String> {
    let schema_size = serializer.serialize(&bytes.len());
    let mut bytes_written = file
        .write_at(position, &schema_size)
        .map_err(|err| format!("{}", err))?;
    bytes_written += file
        .write_at(position + (bytes_written as u64), &bytes)
        .map_err(|err| format!("{}", err))?;

    Ok(bytes_written)
}

fn generate_empty_slots(num_slots: usize) -> Vec<bool> {
    let mut slots = vec![];
    for _ in 0..num_slots {
        slots.push(true);
    }

    slots
}
