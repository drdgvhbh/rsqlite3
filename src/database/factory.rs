use super::{
    io::Pager,
    table::Table,
    {Schema, Serializer},
};
use positioned_io_preview::RandomAccessFile;
use std::fs::{File, OpenOptions};
use std::sync::{Arc, Mutex};

pub struct FactoryConfiguration<S: Serializer> {
    pub database_dir: String,
    pub table_file_ext: String,
    pub page_byte_size: usize,
    pub serializer: S,
}

pub struct Factory<S: Serializer> {
    conf: FactoryConfiguration<S>,
}

impl<S: Serializer> Factory<S> {
    pub fn new(conf: FactoryConfiguration<S>) -> Factory<S> {
        Factory { conf }
    }

    pub fn load_table_from_file(&self, file: File) -> Result<Table<Pager<S>>, String> {
        let ra_file = RandomAccessFile::try_new(file).map_err(|err| format!("{}", err))?;

        let pager = Pager::load_from(ra_file, self.conf.serializer.clone())?;
        let schema = pager.schema().clone();
        Table::new(schema, pager)
    }
}

impl<S: Serializer> super::Factory<Table<Pager<S>>> for Factory<S> {
    fn new_table(&self, schema: Schema) -> Result<Table<Pager<S>>, String> {
        let file_name = format!(
            "{}/{}.{}",
            self.conf.database_dir, schema.table_name, self.conf.table_file_ext
        );
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_name)
            .map_err(|err| format!("{}", err))?;
        let ra_file = RandomAccessFile::try_new(file).map_err(|err| format!("{}", err))?;

        let pager = Pager::new(
            ra_file,
            &schema,
            self.conf.page_byte_size,
            self.conf.serializer.clone(),
        )?;

        Table::new(schema, pager)
    }
}

//
