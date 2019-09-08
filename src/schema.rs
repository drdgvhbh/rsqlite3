use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum Datatype {
    Integer = 1,
}

#[derive(Serialize, Deserialize, Debug)]
struct Column<'a> {
    name: &'a str,
    datatype: Datatype,
}

#[derive(Serialize, Deserialize, Debug)]
struct Table<'a> {
    name: &'a str,
    columns: Vec<Column<'a>>,
}
