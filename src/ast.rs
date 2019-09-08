use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Ast {
    Exit,
    Create(Table),
    Insert(Insertion),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Datatype {
    Integer = 1,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Column {
    pub name: Option<String>,
    pub datatype: Datatype,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Insertion {
    pub column_names: Option<Vec<String>>,
    pub values: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    Integer(i64),
}
