use crate::ast::{Column, Value};
use crate::executor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::IntoIterator;
use std::iter::Iterator;

mod bptree;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IndexedColumn {
    column: Column,
    index: usize,
}

impl executor::Column for IndexedColumn {
    fn name(&self) -> &String {
        &self.column.name
    }
}

pub trait BPTree: IntoIterator<Item = Vec<Value>> + Clone {
    fn insert(&mut self, key: Value, value: Vec<Value>) -> Result<(), String>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Table<T: BPTree> {
    pub name: String,
    rows: T,
    columns: HashMap<String, IndexedColumn>,
}

impl<T: BPTree + 'static> executor::Table for Table<T> {
    fn select_rows(&self) -> Result<Box<dyn Iterator<Item = Vec<Value>>>, String> {
        self.select_rows()
    }
    fn select_rows_with_named_columns(
        &self,
        column_names: &Vec<String>,
    ) -> Result<Box<dyn Iterator<Item = Vec<Value>>>, String> {
        self.select_rows_with_named_columns(column_names)
    }
    fn insert_row(&mut self, row: Vec<Value>) -> Result<&mut dyn executor::Table, String> {
        self.insert_row(row)
            .map(|table| table as &mut dyn executor::Table)
    }

    fn insert_row_with_named_columns(
        &mut self,
        row: HashMap<String, Value>,
    ) -> Result<&mut dyn executor::Table, String> {
        self.insert_row_with_named_columns(row)
            .map(|table| table as &mut dyn executor::Table)
    }

    fn row_len(&self) -> usize {
        self.row_len()
    }

    fn name(&self) -> String {
        return self.name.clone();
    }

    fn columns(&self) -> Vec<Box<dyn executor::Column>> {
        self.columns()
    }
}

impl<T: BPTree + 'static> Table<T> {
    pub fn new<'a, I>(table_name: &str, columns: I, rows: T) -> Result<Table<T>, String>
    where
        I: IntoIterator<Item = &'a Column>,
    {
        let mut verified_columns = HashMap::new();
        for (i, column) in columns.into_iter().enumerate() {
            let column_name = &column.name;
            if verified_columns.contains_key(column_name) {
                return Err(format!("duplicate column name: {}", column_name));
            }
            verified_columns.insert(
                column_name.clone(),
                IndexedColumn {
                    column: column.clone(),
                    index: i,
                },
            );
        }
        return Ok(Table {
            name: table_name.to_lowercase(),
            rows,
            columns: verified_columns,
        });
    }
    pub fn select_rows(&self) -> Result<Box<dyn Iterator<Item = Vec<Value>>>, String> {
        return Ok(Box::new(self.rows.clone().into_iter()));
    }
    pub fn select_rows_with_named_columns(
        &self,
        column_names: &Vec<String>,
    ) -> Result<Box<dyn Iterator<Item = Vec<Value>>>, String> {
        for column_name in column_names {
            if self.columns.get(column_name).is_none() {
                return Err(format!("no such column: {}", column_name));
            }
        }
        let mut indices = Vec::new();
        let result = self.indices(&column_names, &mut indices);
        if result.is_err() {
            return Err(result.unwrap_err());
        }

        return Ok(Box::new(
            self.rows
                .clone()
                .into_iter()
                .map(move |row| {
                    let mut filtered_row = vec![];
                    for i in &indices {
                        filtered_row.push(row[*i].clone())
                    }
                    filtered_row
                })
                .into_iter(),
        ));
    }
    pub fn columns(&self) -> Vec<Box<dyn executor::Column>> {
        let mut columns = vec![];
        for pair in &self.columns {
            let (_, column) = pair;
            columns.push(Box::new(column.clone()) as Box<dyn executor::Column>);
        }

        columns
    }
    pub fn insert_row(&mut self, row: Vec<Value>) -> Result<&mut Table<T>, String> {
        if row.len() != self.row_len() {
            return Err(self.wrong_num_of_columns_error(row.len()));
        }

        self.rows.insert(row[0].clone(), row)?;

        Ok(self)
    }

    fn insert_row_with_named_columns(
        &mut self,
        row: HashMap<String, Value>,
    ) -> Result<&mut Table<T>, String> {
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

        self.rows.insert(row_vec[0].clone(), row_vec)?;

        Ok(self)
    }

    fn indices(&self, column_names: &Vec<String>, dst: &mut Vec<usize>) -> Result<(), String> {
        for column_name in column_names {
            if !self.columns.contains_key(column_name) {
                return Err(format!(
                    "table {} has no column named {}",
                    self.name, column_name
                ));
            }
            dst.push(self.columns.get(column_name).unwrap().index);
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
        return self.columns.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct MockBpTree {}

    impl MockBpTree {
        fn new() -> MockBpTree {
            MockBpTree {}
        }
    }

    impl BPTree for MockBpTree {
        fn insert(&mut self, key: Value, value: Vec<Value>) -> Result<(), String> {
            panic!("not implemented")
        }
    }

    impl IntoIterator for MockBpTree {
        type Item = Vec<Value>;
        type IntoIter = ::std::vec::IntoIter<Self::Item>;
        fn into_iter(self) -> Self::IntoIter {
            panic!("not implemented")
        }
    }

    #[test]
    fn new_tables_should_not_have_duplicate_column_names() {
        let result = Table::new(
            "animals",
            vec![Column::new("feet", false), Column::new("feet", false)].iter(),
            MockBpTree::new(),
        );
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn rows_with_wrong_column_size_should_fail_to_be_inserted() {
        let mut table = Table::new(
            "animals",
            vec![Column::new("feet", false), Column::new("eyes", false)].iter(),
            MockBpTree::new(),
        )
        .unwrap();
        let result = table.insert_row(vec![Value::Integer(49)]);
        assert_eq!(result.is_err(), true);

        let mut row = HashMap::new();
        row.insert("feet".to_string(), Value::Integer(4));
        row.insert("eyes".to_string(), Value::Integer(2));
        row.insert("heart".to_string(), Value::Integer(1));
        let result = table.insert_row_with_named_columns(row);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn rows_with_extraneous_column_name_should_fail_to_be_inserted() {
        let mut table = Table::new(
            "animals",
            vec![Column::new("feet", false)].iter(),
            MockBpTree::new(),
        )
        .unwrap();

        let mut row = HashMap::new();
        row.insert("eyes".to_string(), Value::Integer(2));
        let result = table.insert_row_with_named_columns(row);
        assert_eq!(result.is_err(), true);
    }
}
