use crate::ast::Table;
use std::collections::HashMap;

pub struct Executor {
    table_definitions: HashMap<String, Table>,
}

pub fn new_executor() -> Executor {
    return Executor {
        table_definitions: HashMap::new(),
    };
}

impl Executor {
    pub fn add_table(&mut self, table: &Table) -> Result<(), String> {
        if self.table_definitions.contains_key(&table.name) {
            return Err(format!("table {} already exists", table.name).to_string());
        }
        self.table_definitions
            .insert(table.name.to_lowercase(), table.clone());

        Ok(())
    }
}
