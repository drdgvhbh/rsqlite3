use super::{table, Schema, Serializer, TableValue};
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

    pub fn write_header(&mut self, schema: Schema) -> Result<Schema, String> {
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
        self.write(0, &page_header_bytes)?;

        Ok(page_header.schema)
    }

    /// Writes bytes into a file at position
    ///
    /// ```
    /// +--------+~~~~~~~~~~~~~~~+~~~~~~~~~~~~+
    /// | size   |  size_size    |  bytes     |
    /// | 1 byte |  1-255 bytes  |  2^8 bytes |
    /// +--------+~~~~~~~~~~~~~~~+~~~~~~~~~~~~+
    /// ```
    fn write(&mut self, position: u64, bytes: &[u8]) -> Result<usize, String> {
        let serializer = &self.serializer;
        let file = &mut self.file;
        let size = serializer.serialize(&bytes.len());
        let size_size = serializer.serialize(&size.len());
        debug_assert!(size_size.len() <= std::u8::MAX as usize);
        let mut bytes_written = file
            .write_at(position, &size_size)
            .map_err(|err| format!("{}", err))?;
        bytes_written += file
            .write_at(position + (bytes_written as u64), &size)
            .map_err(|err| format!("{}", err))?;
        bytes_written += file
            .write_at(position + (bytes_written as u64), &bytes)
            .map_err(|err| format!("{}", err))?;

        Ok(bytes_written)
    }
}
