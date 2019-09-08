use crate::ast::Table;
use std::collections::HashMap;

pub struct Executor {
    tables: HashMap<String, Table>,
}

pub fn new_executor() -> Executor {
    return Executor {
        tables: HashMap::new(),
    };
}

impl Executor {
    pub fn add_table(&mut self, table: &Table) -> Result<(), String> {
        if self.tables.contains_key(&table.name) {
            return Err(format!("table {} already exists", table.name).to_string());
        }
        self.tables.insert(table.name.to_lowercase(), table.clone());

        Ok(())
    }
}
