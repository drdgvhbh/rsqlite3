use serde::{Deserialize, Serialize};

pub trait Serializer {
    fn size(&self, obj: impl Serialize) -> usize;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    Boolean,
    Char(usize),
    Int,
    Real,
}

impl DataType {
    pub fn size(&self, serializer: impl Serializer) -> usize {
        match self {
            DataType::Boolean => serializer.size(false),
            DataType::Char(length) => {
                let mut dummy_string = String::new();
                for _ in 0..*length {
                    dummy_string.push('A');
                }
                serializer.size(dummy_string)
            }
            DataType::Int => serializer.size(std::i32::MAX),
            DataType::Real => serializer.size(std::f32::MAX),
        }
    }
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
}
