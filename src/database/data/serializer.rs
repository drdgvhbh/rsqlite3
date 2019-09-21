use super::{Column, DataType, TableSerializationSize, TableValue};
use rmp_serde as rmps;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct MessagePackSerializer {}

impl super::Serializer for MessagePackSerializer {
    fn size(&self, columns: &Vec<Column>) -> TableSerializationSize {
        let mut dummy_input = vec![];
        for column in columns {
            match column.datatype {
                DataType::Boolean => dummy_input.push(TableValue::Boolean(true)),
                DataType::Char(size) => {
                    let mut dummy_str = String::new();
                    for _ in 0..size {
                        dummy_str.push('a');
                    }

                    dummy_input.push(TableValue::Char(dummy_str))
                }
                DataType::Int => dummy_input.push(TableValue::Int(std::i32::MAX)),
                DataType::Real => dummy_input.push(TableValue::Real(std::f32::MAX)),
            }
        }

        let row_size = rmps::to_vec(&dummy_input).unwrap().len();

        TableSerializationSize {
            row_size,
            // https://github.com/msgpack/msgpack/blob/master/spec.md#array-format-family
            vector_size: 5,
        }
    }

    fn serialize<S: Serialize>(&self, obj: &S) -> Vec<u8> {
        rmps::to_vec(&obj).unwrap()
    }

    fn deserialize<D: DeserializeOwned>(&self, obj: &[u8]) -> Result<D, String> {
        rmps::from_slice::<D>(&obj).map_err(|err| format!("{}", err))
    }
}
