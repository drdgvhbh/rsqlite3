use crate::ast::Value;
use rmp_serde;
use std::io::{Read, Seek, Write};

pub struct Pager<F: Write + Read + Seek> {
    transaction_log: F,
    file: F,
    page_size: u16,
    page_cache: lru::LruCache<u32, Vec<Value>>,
    num_pages_on_disk: u32,
}

impl<F: Write + Read + Seek> Pager<F> {
    pub fn new(
        transaction_log: F,
        file: F,
        page_cache: lru::LruCache<u32, Vec<Value>>,
        page_size: u16,
    ) -> Pager<F> {
        return Pager {
            transaction_log,
            file,
            page_size,
            page_cache,
            num_pages_on_disk: 0,
        };
    }

    pub fn insert(&self, row: &[Value]) -> u32 {
        let serialized = rmp_serde::to_vec(&row).unwrap();
        println!("{:#?}", serialized);
        0
    }
}
