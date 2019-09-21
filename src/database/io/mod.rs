use super::{table, RecordID, Schema, Serializer, TableValue};
use positioned_io_preview::RandomAccessFile;
use positioned_io_preview::ReadAt;
use positioned_io_preview::WriteAt;
use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Debug, Serialize, Deserialize)]
pub struct PageHeader {
    schema: Schema,
    num_pages: usize,
    page_byte_size: usize,
    page_capacity: usize,
    free_pages: BinaryHeap<usize>,
}

type PageNumber = usize;

pub struct Pager<S: Serializer> {
    file: RandomAccessFile,
    page_header: PageHeader,
    pages: HashMap<PageNumber, Vec<Option<Vec<TableValue>>>>,
    serializer: S,
}

impl<S: Serializer> table::Pager for Pager<S> {
    fn has_free_pages(&self) -> Result<bool, String> {
        Ok(self.page_header.free_pages.peek().is_none())
    }

    fn allocate_page(&mut self) -> Result<(), String> {
        let new_page_number = self.page_header.num_pages + 1;
        let slots = vec![false; self.page_header.page_capacity];
        let position = new_page_number * self.page_header.page_byte_size;

        self.write(
            position.try_into().unwrap(),
            &self.serializer.serialize(&slots),
        )?;

        let page_number = self.page_header.num_pages + 1;
        self.page_header.num_pages = page_number;
        self.page_header.free_pages.push(page_number);
        self.pages
            .insert(page_number, vec![None; self.page_header.page_capacity]);

        Ok(())
    }

    fn insert(&mut self, row: Vec<TableValue>) -> Result<RecordID, String> {
        if self.page_header.free_pages.peek().is_none() {
            return Err("table is full; allocate a new page".into());
        }
        let page_number = self.page_header.free_pages.peek().unwrap();

        let empty_slot_idx = self
            .pages
            .get(&page_number)
            .unwrap()
            .iter()
            .enumerate()
            .find(|&r| r.1.is_none())
            .unwrap()
            .0;

        let page = self.pages.get_mut(page_number).unwrap();
        page[empty_slot_idx] = Some(row);

        Ok(RecordID::new(
            (*page_number).try_into().unwrap(),
            empty_slot_idx.try_into().unwrap(),
        ))
    }
}

impl<S: Serializer> Pager<S> {
    pub fn new(
        mut file: RandomAccessFile,
        schema: &Schema,
        page_byte_size: usize,
        serializer: S,
    ) -> Result<Pager<S>, String> {
        let page_header = write_header(&mut file, schema, page_byte_size, &serializer)?;
        Ok(Pager {
            file,
            pages: HashMap::new(),
            page_header,
            serializer,
        })
    }

    pub fn read_header(&self) -> Result<PageHeader, String> {
        let mut size = [0; 1];
        let mut bytes_read = self
            .file
            .read_at(0, &mut size)
            .map_err(|err| format!("{}", err))?;
        let mut size_size = vec![0; size[0] as usize];
        bytes_read += self
            .file
            .read_at(bytes_read.try_into().unwrap(), &mut size_size)
            .map_err(|err| format!("{}", err))?;
        let header_size: u8 = self.serializer.deserialize(&size_size)?;

        let mut page_header_bytes = vec![0; header_size as usize];
        self.file
            .read_at(bytes_read.try_into().unwrap(), &mut page_header_bytes)
            .map_err(|err| format!("{}", err))?;

        self.serializer.deserialize(&page_header_bytes)
    }

    fn write(&mut self, position: u64, bytes: &[u8]) -> Result<usize, String> {
        write(&mut self.file, position, bytes, &self.serializer)
    }
}

fn write_header<S: Serializer>(
    file: &mut RandomAccessFile,
    schema: &Schema,
    page_byte_size: usize,
    serializer: &S,
) -> Result<PageHeader, String> {
    let tss = schema.size(serializer);
    let mut page_capacity = page_byte_size / tss.row_size;
    let mut slot_capacity_bytes = serializer.serialize(&page_capacity);
    let mut slots_bytes = serializer.serialize(&vec![false; page_capacity]);

    while 1
        + slot_capacity_bytes.len()
        + slots_bytes.len()
        + tss.vector_size
        + tss.row_size * page_capacity
        > page_byte_size
    {
        page_capacity -= 1;
        slot_capacity_bytes = serializer.serialize(&page_capacity);
        slots_bytes = serializer.serialize(&vec![false; page_capacity]);
    }

    let page_header = PageHeader {
        schema: schema.clone(),
        num_pages: 0,
        page_byte_size: page_byte_size,
        page_capacity,
        free_pages: BinaryHeap::new(),
    };
    let page_header_bytes = serializer.serialize(&page_header);

    if page_header_bytes.len() > page_byte_size {
        return Err("page size is not large enough to fight page header".to_string());
    }
    write(file, 0, &page_header_bytes, serializer)?;

    Ok(page_header)
}

/// Writes bytes into a file at position
///
/// ```
/// +--------+~~~~~~~~~~~~~~~+~~~~~~~~~~~~+
/// | size   |  size_size    |  bytes     |
/// | 1 byte |  1-255 bytes  |  2^8 bytes |
/// +--------+~~~~~~~~~~~~~~~+~~~~~~~~~~~~+
/// ```
fn write<S: Serializer>(
    file: &mut RandomAccessFile,
    position: u64,
    bytes: &[u8],
    serializer: &S,
) -> Result<usize, String> {
    let serializer = &serializer;
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
