use crate::ast::{Insertion, TableSchema, Value, Datatype};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap};



#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Table {
    name: String,
    column_datatypes: Vec<Datatype>,
    rows: Vec<Vec<Value>>,
    column_names: HashMap<String, u16>
}

pub struct Executor {
    tables: HashMap<String, Table>,
}

pub fn new_executor() -> Executor {
    return Executor {
        tables: HashMap::new(),
    };
}

impl Executor {
    pub fn add_table(&mut self, schema: &TableSchema) -> Result<(), String> {
        if self.tables.contains_key(&schema.name) {
            return Err(format!("table {} already exists", schema.name).to_string());
        }
        let column_datatypes = vec![];
        let column_names = HashMap::new();
        for (i, column) in schema.columns.iter().enumerate() {
            column_datatypes.push(column.datatype);
            match column.name {
                None => {},
                Some(column_name) => { column_names.insert(column_name, i as u16); }
            }
        } 
        self.tables.insert(
            schema.name.to_lowercase(),
            Table {
                name: schema.name,
                column_datatypes,
                column_names,
                rows: vec![],
            },
        );

        Ok(())
    }
}
