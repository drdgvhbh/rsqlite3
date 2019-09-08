use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum Ast {
    Exit,
    Create(Table),
    Insert(Insertion),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Datatype {
    Integer = 1,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Column {
    pub name: Option<String>,
    pub datatype: Datatype,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Insertion {
    pub column_names: Option<Vec<String>>,
    pub values: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
}
