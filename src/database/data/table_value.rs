use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableValue {
    Boolean(bool),
    Char(String),
    Int(i32),
    Real(f32),
}

/* impl Serialize for TableValue {
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
 */
/* impl<'de> Deserialize<'de> for TableValue {
    fn deserialize<D>(deserializer: D) -> Result<TableValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(visitor: V)(I32Visitor)
    }
}
 */
