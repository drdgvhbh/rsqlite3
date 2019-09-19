use super::{
    io::{Page, Pager},
    table::Table,
    {Schema, Serializer},
};
use positioned_io_preview::RandomAccessFile;
use std::fs::File;

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
}

impl<S: Serializer> super::Factory<Table<Page, Pager<S>>> for Factory<S> {
    fn new_table(&self, schema: Schema) -> Result<Table<Page, Pager<S>>, String> {
        let file_name = format!(
            "{}/{}.{}",
            self.conf.database_dir, schema.table_name, self.conf.table_file_ext
        );
        let file = File::create(file_name).map_err(|err| format!("{}", err))?;
        let ra_file = RandomAccessFile::try_new(file).map_err(|err| format!("{}", err))?;

        let mut pager = Pager::new(
            ra_file,
            self.conf.page_byte_size,
            self.conf.serializer.clone(),
        );
        pager.write_header(schema)?;
        panic!("TODO")
    }
}
