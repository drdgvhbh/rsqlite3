use crate::ast::{Column, Datatype, Value};
use crate::executor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::slice;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Table {
    pub name: String,
    column_datatypes: Vec<Datatype>,
    rows: Vec<Box<Vec<Value>>>,
    column_names: HashMap<String, usize>,
}

pub fn new_table(table_name: &str, columns: slice::Iter<Column>) -> Result<Box<Table>, String> {
    let mut column_datatypes = vec![];
    let mut column_names = HashMap::new();
    for (i, column) in columns.enumerate() {
        column_datatypes.push(column.datatype.clone());
        match &column.name {
            None => {}
            Some(column_name) => {
                if column_names.contains_key(column_name) {
                    return Err(format!("duplicate column name: {}", column_name));
                }
                column_names.insert(column_name.clone(), i);
            }
        }
    }
    return Ok(Box::new(Table {
        name: table_name.to_lowercase(),
        column_datatypes,
        rows: vec![],
        column_names,
    }));
}

impl executor::Table for Table {
    fn insert_row(&mut self, row: slice::Iter<Value>) -> Result<(), String> {
        if row.len() != self.row_len() {
            return Err(self.wrong_num_of_columns_error(row.len()));
        }

        let row_vec = row.map(|value| value.clone()).collect();
        self.rows.push(Box::new(row_vec));

        Ok(())
    }

    fn insert_row_with_named_columns(
        &mut self,
        row: &HashMap<String, Value>,
    ) -> Result<(), String> {
        if row.len() > self.row_len() {
            return Err(self.wrong_num_of_columns_error(row.len()));
        }

        let mut indices = vec![];
        let column_names = row.keys().map(|k| k.clone()).collect();
        let result = self.indices(&column_names, &mut indices);
        if result.is_err() {
            return result;
        }

        let mut row_vec = vec![Value::Null; self.row_len()];
        for kv in indices.iter().zip(row.values()) {
            let (index, value) = kv;
            row_vec[*index] = value.clone();
        }
        self.rows.push(Box::new(row_vec));

        Ok(())
    }

    fn row_len(&self) -> usize {
        return self.row_len();
    }
}

impl Table {
    fn indices(&self, column_names: &Vec<String>, dst: &mut Vec<usize>) -> Result<(), String> {
        for column_name in column_names {
            if !self.column_names.contains_key(column_name) {
                return Err(format!(
                    "table {} has no column named {}",
                    self.name, column_name
                ));
            }
            dst.push(*self.column_names.get(column_name).unwrap());
        }

        Ok(())
    }

    fn wrong_num_of_columns_error(&self, num_columns: usize) -> String {
        return format!(
            "table {} has {} columns but {} values were supplied",
            self.name,
            num_columns,
            self.row_len()
        );
    }

    fn row_len(&self) -> usize {
        return self.column_datatypes.len();
    }
}
