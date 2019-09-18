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

pub struct Schema {
    pub table_name: String,
}

impl Schema {
    pub fn new(table_name: &str) -> Schema {
        Schema {
            table_name: table_name.to_string(),
        }
    }
}
