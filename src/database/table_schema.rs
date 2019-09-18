#[derive(Debug)]
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
