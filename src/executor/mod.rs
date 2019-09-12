use self::table::new_table;
use crate::ast::{Insertion, TableSchema, Value};
use std::collections::HashMap;
use std::slice;

mod table;

pub trait Table {
    fn insert_row(&mut self, row: slice::Iter<Value>) -> Result<(), String>;
    fn insert_row_with_named_columns(&mut self, row: &HashMap<String, Value>)
        -> Result<(), String>;
    fn row_len(&self) -> usize;
}

pub struct Executor {
    tables: Box<HashMap<String, Box<dyn Table>>>,
}

pub fn new_executor() -> Executor {
    return Executor {
        tables: Box::new(HashMap::new()),
    };
}

impl Executor {
    pub fn add_table(&mut self, schema: &TableSchema) -> Result<(), String> {
        let table_name = &schema.name;
        if self.table_exists(&table_name) {
            return Err(format!("table {} already exists", schema.name).to_string());
        }
        let result = new_table(&table_name, schema.columns.iter());
        match result {
            Err(err) => Err(err),
            Ok(table) => {
                self.tables.insert(table_name.to_string(), table);
                Ok(())
            }
        }
    }

    pub fn insert(&mut self, insertion: &Insertion) -> Result<(), String> {
        let table_name = &insertion.table_name;
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
            let result = table.insert_row(values);
            if result.is_err() {
                return result;
            }
        } else {
            let column_names = insertion.column_names().unwrap();
            let mut row = HashMap::new();
            for kv in column_names.zip(values) {
                let (column_name, value) = kv;
                row.insert(column_name.clone(), value.clone());
            }
            let result = table.insert_row_with_named_columns(&row);
            if result.is_err() {
                return result;
            }
        }
        Ok(())
    }

    fn table_exists(&self, table_name: &str) -> bool {
        return self.tables.get(table_name).is_some();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use std::collections::HashMap;

    #[test]
    fn should_fail_to_create_a_table_if_one_with_same_name_already_exists() {
        let table_name = "apples";
        let mut tables: Box<HashMap<String, Box<dyn Table>>> = Box::new(HashMap::new());
        tables.insert(
            table_name.to_string(),
            table::new_table(table_name, vec![].iter()).unwrap(),
        );
        let mut executor = Executor { tables };
        let result = executor.add_table(&TableSchema {
            name: table_name.to_string(),
            columns: vec![],
        });
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn should_fail_to_insert_row_if_table_does_not_exist() {
        let table_name = "oranges".to_string();
        let mut executor = Executor {
            tables: Box::new(HashMap::new()),
        };

        let result = executor.insert(&new_insertion(&table_name, None, vec![]));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn should_fail_to_insert_row_if_row_has_more_columns_than_table() {
        let table_name = "peaches".to_string();
        let mut tables: Box<HashMap<String, Box<dyn Table>>> = Box::new(HashMap::new());
        tables.insert(
            table_name.to_string(),
            new_table(
                &table_name,
                vec![Column {
                    name: None,
                    datatype: Datatype::Integer,
                }]
                .iter(),
            )
            .unwrap(),
        );
        let mut executor = Executor { tables };
        let result = executor.insert(&new_insertion(
            &table_name,
            None,
            vec![Value::Integer(32), Value::Integer(1337)],
        ));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn should_fail_to_insert_row_if_num_values_exceeds_num_column_names() {
        let table_name = "strawberries".to_string();
        let mut tables: Box<HashMap<String, Box<dyn Table>>> = Box::new(HashMap::new());
        tables.insert(
            table_name.to_string(),
            new_table(
                &table_name,
                vec![Column {
                    name: Some("a".to_string()),
                    datatype: Datatype::Integer,
                }]
                .iter(),
            )
            .unwrap(),
        );
        let mut executor = Executor { tables };
        let result = executor.insert(&new_insertion(
            &table_name,
            Some(vec!["a".to_string()]),
            vec![Value::Integer(32), Value::Integer(1337)],
        ));
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn should_fail_to_insert_row_if_one_of_the_column_names_does_not_exist() {
        let table_name = "mud".to_string();
        let mut tables: Box<HashMap<String, Box<dyn Table>>> = Box::new(HashMap::new());
        let mut column_names = HashMap::new();
        column_names.insert("z".to_string(), 0);
        tables.insert(
            table_name.to_string(),
            new_table(
                &table_name,
                vec![Column {
                    name: None,
                    datatype: Datatype::Integer,
                }]
                .iter(),
            )
            .unwrap(),
        );
        let mut executor = Executor { tables };
        let result = executor.insert(&new_insertion(
            &table_name,
            Some(vec!["a".to_string()]),
            vec![Value::Integer(32)],
        ));
        assert_eq!(result.is_err(), true);
    }
}
