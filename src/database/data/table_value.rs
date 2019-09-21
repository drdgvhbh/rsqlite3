use serde::{Serialize, Serializer};

#[derive(Debug, Clone)]
pub enum TableValue {
    Boolean(bool),
    Char(String),
    Int(i32),
    Real(f32),
}

impl Serialize for TableValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TableValue::Boolean(b) => serializer.serialize_bool(*b),
            TableValue::Char(s) => serializer.serialize_str(s),
            TableValue::Int(i) => serializer.serialize_i32(*i),
            TableValue::Real(f) => serializer.serialize_f32(*f),
        }
    }
}
