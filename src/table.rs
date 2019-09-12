use crate::ast::{Column, Datatype, Value};
use crate::executor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::ExactSizeIterator;
use std::iter::Iterator;
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
    fn insert_row(&mut self, row: slice::Iter<Value>) -> Result<&mut dyn executor::Table, String> {
        self.insert_row(row)
            .map(|table| table as &mut dyn executor::Table)
    }

    fn insert_row_with_named_columns(
        &mut self,
        row: &HashMap<String, Value>,
    ) -> Result<&mut dyn executor::Table, String> {
        self.insert_row_with_named_columns(row)
            .map(|table| table as &mut dyn executor::Table)
    }

    fn row_len(&self) -> usize {
        self.row_len()
    }

    fn name(&self) -> &String {
        return &self.name;
    }

    fn columns(&self) -> Box<[(String, Datatype)]> {
        self.columns()
    }
}

impl Table {
    pub fn columns(&self) -> Box<[(String, Datatype)]> {
        let mut columns = vec![];
        for pair in &self.column_names {
            let (column_name, i) = pair;
            columns.push((column_name.clone(), self.column_datatypes[*i].clone()));
        }

        columns.into_boxed_slice()
    }
    pub fn insert_row(&mut self, row: slice::Iter<Value>) -> Result<&mut Table, String> {
        if row.len() != self.row_len() {
            return Err(self.wrong_num_of_columns_error(row.len()));
        }

        let row_vec = row.map(|value| value.clone()).collect();
        self.rows.push(Box::new(row_vec));

        Ok(self)
    }

    fn insert_row_with_named_columns(
        &mut self,
        row: &HashMap<String, Value>,
    ) -> Result<&mut Table, String> {
        if row.len() > self.row_len() {
            return Err(self.wrong_num_of_columns_error(row.len()));
        }

        let mut indices = vec![];
        let column_names = row.keys().map(|k| k.clone()).collect();
        let result = self.indices(&column_names, &mut indices);
        if result.is_err() {
            return result.map(|_| self);
        }

        let mut row_vec = vec![Value::Null; self.row_len()];
        for kv in indices.iter().zip(row.values()) {
            let (index, value) = kv;
            row_vec[*index] = value.clone();
        }
        self.rows.push(Box::new(row_vec));

        Ok(self)
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tables_should_not_have_duplicate_column_names() {
        let result = new_table(
            "animals",
            vec![
                Column {
                    name: Some("feet".to_string()),
                    datatype: Datatype::Integer,
                },
                Column {
                    name: Some("feet".to_string()),
                    datatype: Datatype::Integer,
                },
            ]
            .iter(),
        );
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn rows_with_wrong_column_size_should_fail_to_be_inserted() {
        let mut table = new_table(
            "animals",
            vec![
                Column {
                    name: Some("feet".to_string()),
                    datatype: Datatype::Integer,
                },
                Column {
                    name: Some("eyes".to_string()),
                    datatype: Datatype::Integer,
                },
            ]
            .iter(),
        )
        .unwrap();
        let result = table.insert_row(vec![Value::Integer(49)].iter());
        assert_eq!(result.is_err(), true);

        let mut row = HashMap::new();
        row.insert("feet".to_string(), Value::Integer(4));
        row.insert("eyes".to_string(), Value::Integer(2));
        row.insert("heart".to_string(), Value::Integer(1));
        let result = table.insert_row_with_named_columns(&row);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn rows_with_extraneous_column_name_should_fail_to_be_inserted() {
        let mut table = new_table(
            "animals",
            vec![Column {
                name: Some("feet".to_string()),
                datatype: Datatype::Integer,
            }]
            .iter(),
        )
        .unwrap();

        let mut row = HashMap::new();
        row.insert("eyes".to_string(), Value::Integer(2));
        let result = table.insert_row_with_named_columns(&row);
        assert_eq!(result.is_err(), true);
    }
}
