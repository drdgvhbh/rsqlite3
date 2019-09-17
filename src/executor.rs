use crate::ast::{ColumnSet, Value};
use std::collections::HashMap;

#[cfg(test)]
extern crate mockers_derive;

#[cfg(test)]
use mockers_derive::mocked;

pub trait Column {
    fn name(&self) -> &String;
}

#[cfg_attr(test, mocked)]
pub trait Table {
    /// Name of the table
    ///
    /// Returns a String rather than reference because the mocker does not
    /// support references
    fn name(&self) -> String;
    fn insert_row(&mut self, row: Vec<Value>) -> Result<&mut dyn Table, String>;
    fn insert_row_with_named_columns(
        &mut self,
        row: HashMap<String, Value>,
    ) -> Result<&mut dyn Table, String>;
    fn row_len(&self) -> usize;
    fn select_rows(&self) -> Result<Box<dyn Iterator<Item = Vec<Value>>>, String>;
    fn select_rows_with_named_columns(
        &self,
        column_names: &Vec<String>,
    ) -> Result<Box<dyn Iterator<Item = Vec<Value>>>, String>;
    fn columns(&self) -> Vec<Box<dyn Column>>;
}

pub trait Insertion {
    fn table_name(&self) -> &String;
    fn validate(&self) -> Result<(), String>;
    fn column_names(&self) -> Option<Box<dyn Iterator<Item = String>>>;
    fn values(&self) -> Box<dyn Iterator<Item = Value>>;
}

pub trait Selection {
    fn table_name(&self) -> &String;
    fn validate(&self) -> Result<(), String>;
    fn columns(&self) -> ColumnSet;
}

pub struct Executor<T: Table> {
    tables: HashMap<String, T>,
}

impl<T: Table> Executor<T> {
    pub fn new() -> Executor<T> {
        return Executor {
            tables: HashMap::new(),
        };
    }

    pub fn add_table(&mut self, table: T) -> Result<(), String> {
        let table_name = table.name();
        if self.table_exists(&table_name) {
            return Err(format!("table {} already exists", &table_name).to_string());
        }
        self.tables.insert(table_name.to_string(), table);
        Ok(())
    }

    pub fn insert<I: Insertion>(&mut self, insertion: I) -> Result<(), String> {
        let table_name = insertion.table_name();
        if !self.table_exists(table_name) {
            return Err(format!("no such table: {}", table_name));
        }
        let result = insertion.validate();
        if result.is_err() {
            return result;
        }
        let table = self.tables.get_mut(table_name).unwrap();
        let values = insertion.values();

        if insertion.column_names().is_none() {
            let result = table.insert_row(values.collect());
            if result.is_err() {
                return result.and_then(|_| Ok(()));
            }
        } else {
            let column_names = insertion.column_names().unwrap();
            let mut row = HashMap::new();
            for kv in column_names.zip(values) {
                let (column_name, value) = kv;
                row.insert(column_name.clone(), value.clone());
            }
            let result = table.insert_row_with_named_columns(row);
            if result.is_err() {
                return result.map(|_| ());
            }
        }
        Ok(())
    }

    pub fn select<S: Selection>(
        &self,
        selection: S,
    ) -> Result<Box<dyn Iterator<Item = Vec<Value>>>, String> {
        let table_name = selection.table_name();
        if !self.table_exists(table_name) {
            return Err(format!("no such table: {}", table_name));
        }

        let table = self.tables.get(table_name).unwrap();
        let column_set = &selection.columns();
        match column_set {
            ColumnSet::WildCard => table.select_rows(),
            ColumnSet::Names(column_names) => table.select_rows_with_named_columns(&column_names),
        }
    }

    fn table_exists(&self, table_name: &str) -> bool {
        return self.tables.get(table_name).is_some();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast;
    use mockers::Scenario;
    use std::collections::HashMap;

    #[test]
    fn should_fail_to_create_a_table_if_one_with_same_name_already_exists() {
        let scenario = Scenario::new();
        let table_name = "apples".to_string();
        let (table, _) = scenario.create_mock_for::<dyn Table>();
        let (table2, table2_handle) = scenario.create_mock_for::<dyn Table>();
        scenario.expect(table2_handle.name().and_return("apples".to_string()));
        let mut tables: HashMap<String, TableMock> = HashMap::new();
        tables.insert(table_name, table);

        let mut executor = Executor { tables };
        let result = executor.add_table(table2);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn should_fail_to_insert_row_if_table_does_not_exist() {
        let table_name = "oranges".to_string();
        let mut executor = Executor::<TableMock> {
            tables: HashMap::new(),
        };

        let result = executor.insert(ast::Insertion::new(&table_name, None, vec![]));
        assert_eq!(result.is_err(), true);
    }
}
