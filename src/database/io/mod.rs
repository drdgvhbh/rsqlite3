use super::{table, RecordID, Schema, Serializer, TableValue};
use positioned_io_preview::RandomAccessFile;
use positioned_io_preview::ReadAt;
use positioned_io_preview::WriteAt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::BinaryHeap;
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
    pages: BTreeMap<PageNumber, Vec<Option<Vec<TableValue>>>>,
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
        let free_pages = &mut self.page_header.free_pages;
        let page_number = free_pages.peek().unwrap().clone();

        let page = self.pages.get_mut(&page_number).unwrap();
        let empty_slots = page
            .iter()
            .enumerate()
            .filter(|&r| r.1.is_none())
            .collect::<Vec<_>>();

        let empty_slot_idx = empty_slots[0].0;
        if empty_slots.len() == 1 {
            free_pages.pop();
        }
        page[empty_slot_idx] = Some(row);

        Ok(RecordID::new(
            (page_number).try_into().unwrap(),
            empty_slot_idx.try_into().unwrap(),
        ))
    }

    fn flush(&mut self) -> Result<(), String> {
        let header_bytes = self.serializer.serialize(&self.page_header);
        write(&mut self.file, 0, &header_bytes, &self.serializer)?;

        let page_byte_size = self.page_header.page_byte_size;

        for page in self.pages.iter() {
            let (page_number, rows) = page;
            let serialized_rows = self.serializer.serialize(&rows);
            let mut bytes = vec![0; page_byte_size];
            let position = page_number * page_byte_size;

            self.file
                .write_at(position.try_into().unwrap(), &bytes)
                .map_err(|err| format!("{}", err))?;
            for (i, byte) in serialized_rows.into_iter().enumerate() {
                bytes[i] = byte;
            }
            self.file
                .write_at(position.try_into().unwrap(), &bytes)
                .map_err(|err| format!("{}", err))?;
        }

        Ok(())
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
            pages: BTreeMap::new(),
            page_header,
            serializer,
        })
    }

    pub fn load_from(mut file: RandomAccessFile, serializer: S) -> Result<Pager<S>, String> {
        let bytes = read(&mut file, 0, &serializer)?;
        let page_header: PageHeader = serializer.deserialize(&bytes)?;

        let mut pages = BTreeMap::new();
        for page_number in 1..page_header.num_pages + 1 {
            let position = page_number * page_header.page_byte_size;
            let mut buf = vec![0; page_header.page_byte_size];
            file.read_at(position as u64, &mut buf)
                .map_err(|err| format!("{}", err))?;

            let page: Vec<Option<Vec<TableValue>>> = serializer.deserialize(&buf)?;
            pages.insert(page_number, page);
        }

        Ok(Pager {
            file,
            pages,
            page_header,
            serializer,
        })
    }

    pub fn schema(&self) -> &Schema {
        &self.page_header.schema
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

    while tss.vector_size + tss.row_size * page_capacity > page_byte_size {
        page_capacity -= 1;
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
/// | 1 byte |  1-255 bytes  |  N bytes |
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

fn read<S: Serializer>(
    file: &mut RandomAccessFile,
    position: u64,
    serializer: &S,
) -> Result<Vec<u8>, String> {
    let serializer = &serializer;
    let mut buf = [0; 1];
    let mut bytes_read = file
        .read_at(position, &mut buf)
        .map_err(|err| format!("{}", err))?;
    let size_of_size: usize = serializer.deserialize(&buf)?;

    let mut buf = vec![0; size_of_size];
    bytes_read += file
        .read_at(position + bytes_read as u64, &mut buf)
        .map_err(|err| format!("{}", err))?;
    let size: usize = serializer.deserialize(&buf)?;

    let mut buf = vec![0; size];
    bytes_read += file
        .read_at(position + bytes_read as u64, &mut buf)
        .map_err(|err| format!("{}", err))?;

    Ok(buf)
}
