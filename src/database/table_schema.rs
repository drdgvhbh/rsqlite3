use serde::{Deserialize, Serialize};

pub struct TableSerializationSize {
    /// Number of bytes required to store a single row
    pub row_size: usize,

    /// Number of bytes required to encode a vector that stores a collection of rows
    ///
    /// Example: Message Pack Family of Encoding
    ///
    ///
    /// Array 32 stores an array whose length is up to (2^32)-1 elements:
    /// ```
    /// +--------+--------+--------+--------+--------+~~~~~~~~~~~~~~~~~+
    /// |  0xdd  |ZZZZZZZZ|ZZZZZZZZ|ZZZZZZZZ|ZZZZZZZZ|    N objects    |
    /// +--------+--------+--------+--------+--------+~~~~~~~~~~~~~~~~~+
    /// ```
    /// In this case, the vector size would be 5 bytes
    pub vector_size: usize,
}

pub trait Serializer {
    /// Calculates the size in bytes of a row consisting of these columns
    /// and the size of a collection of the same rows
    fn size(&self, columns: &Vec<Column>) -> TableSerializationSize;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    Boolean,
    Char(usize),
    Int,
    Real,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub datatype: DataType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    pub table_name: String,
    pub columns: Vec<Column>,
}

impl Schema {
    pub fn new(table_name: &str, columns: Vec<Column>) -> Schema {
        Schema {
            table_name: table_name.to_string(),
            columns,
        }
    }

    pub fn size(&self, serializer: impl Serializer) -> TableSerializationSize {
        serializer.size(&self.columns)
    }
}
