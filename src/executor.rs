use crate::ast::{Datatype, Insertion, TableSchema, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Table {
    name: String,
    column_datatypes: Vec<Datatype>,
    rows: Vec<Box<Vec<Option<Value>>>>,
    column_names: HashMap<String, usize>,
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
        let mut column_datatypes = vec![];
        let mut column_names = HashMap::new();
        for (i, column) in schema.columns.iter().enumerate() {
            column_datatypes.push(column.datatype.clone());
            match &column.name {
                None => {}
                Some(column_name) => {
                    column_names.insert(column_name.clone(), i);
                }
            }
        }
        self.tables.insert(
            schema.name.to_lowercase(),
            Table {
                name: schema.name.clone(),
                column_datatypes,
                column_names,
                rows: vec![],
            },
        );

        Ok(())
    }

    pub fn insert(&mut self, insertion: &Insertion) -> Result<(), Vec<String>> {
        let table_name = &insertion.table_name;
        let table_opt = self.tables.get_mut(table_name);
        let mut errs = Vec::<String>::new();
        if table_opt.is_none() {
            errs.push(format!("no such table: {}", table_name));
            return Err(errs);
        }
        let table = table_opt.unwrap();
        let mut insertion_indices: Vec<usize> = vec![];
        let values = &insertion.values;

        if insertion.column_names.is_none() {
            for i in 0..values.len() {
                insertion_indices.push(i);
            }
        } else {
            let column_names = insertion.column_names.as_ref().unwrap();
            if values.len() > column_names.len() {
                errs.push(format!(
                    "{} values for {} columns",
                    values.len(),
                    column_names.len()
                ));
            }
            for column_name in column_names {
                let column_idx_opt = table.column_names.get(column_name);
                if column_idx_opt.is_none() {
                    errs.push(format!(
                        "table {} has no column named {}",
                        table_name, column_name
                    ));
                    continue;
                }
                let column_idx = column_idx_opt.unwrap();
                insertion_indices.push(*column_idx);
            }
        }

        if values.len() > table.column_datatypes.len() {
            errs.push(format!(
                "table {} has {} columns but {} values were supplied",
                table_name,
                table.column_datatypes.len(),
                values.len(),
            ));
        }

        if !errs.is_empty() {
            return Err(errs);
        }

        assert_eq!(insertion_indices.len(), values.len());
        let mut row: Vec<Option<Value>> = vec![None; table.column_datatypes.len()];
        for (i, v) in insertion_indices.iter().zip(values) {
            row[*i] = Some(v.clone());
        }
        table.rows.push(Box::new(row));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_fail_to_create_a_table_if_one_with_same_name_already_exists() {
        let table_name = "apples";
        let mut tables = HashMap::new();
        tables.insert(
            table_name.to_string(),
            Table {
                name: table_name.to_string(),
                column_datatypes: vec![],
                rows: vec![],
                column_names: HashMap::new(),
            },
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
            tables: HashMap::new(),
        };
        let result = executor.insert(&Insertion {
            table_name,
            column_names: None,
            values: vec![],
        });
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn should_fail_to_insert_row_if_row_has_more_columns_than_table() {
        let table_name = "peaches".to_string();
        let mut tables = HashMap::new();
        tables.insert(
            table_name.to_string(),
            Table {
                name: table_name.to_string(),
                column_datatypes: vec![Datatype::Integer],
                rows: vec![],
                column_names: HashMap::new(),
            },
        );
        let mut executor = Executor { tables };
        let result = executor.insert(&Insertion {
            table_name,
            column_names: None,
            values: vec![Value::Integer(32), Value::Integer(1337)],
        });
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn should_fail_to_insert_row_if_num_values_exceeds_num_column_names() {
        let table_name = "strawberries".to_string();
        let mut tables = HashMap::new();
        tables.insert(
            table_name.to_string(),
            Table {
                name: table_name.to_string(),
                column_datatypes: vec![Datatype::Integer],
                rows: vec![],
                column_names: HashMap::new(),
            },
        );
        let mut executor = Executor { tables };
        let result = executor.insert(&Insertion {
            table_name,
            column_names: Some(vec!["a".to_string()]),
            values: vec![Value::Integer(32), Value::Integer(1337)],
        });
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn should_fail_to_insert_row_if_one_of_the_column_names_does_not_exist() {
        let table_name = "mud".to_string();
        let mut tables = HashMap::new();
        let mut column_names = HashMap::new();
        column_names.insert("z".to_string(), 0);
        tables.insert(
            table_name.to_string(),
            Table {
                name: table_name.to_string(),
                column_datatypes: vec![Datatype::Integer],
                rows: vec![],
                column_names,
            },
        );
        let mut executor = Executor { tables };
        let result = executor.insert(&Insertion {
            table_name,
            column_names: Some(vec!["a".to_string()]),
            values: vec![Value::Integer(32)],
        });
        assert_eq!(result.is_err(), true);
    }
}
