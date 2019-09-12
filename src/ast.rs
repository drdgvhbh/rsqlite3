use serde::{Deserialize, Serialize};
use std::slice::Iter;

#[derive(Debug, PartialEq)]
pub enum Ast {
    Exit,
    Create(TableSchema),
    Insert(Box<Insertion>),
    Select(Selection),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ColumnSet {
    WildCard,
    Names(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Selection {
    pub table_name: String,
    pub columns: ColumnSet,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Datatype {
    Integer = 1,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Column {
    pub name: Option<String>,
    pub datatype: Datatype,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Insertion {
    pub table_name: String,
    column_names: Option<Vec<String>>,
    values: Box<Vec<Value>>,
}

pub fn new_insertion(
    table_name: &str,
    column_names: Option<Vec<String>>,
    values: Vec<Value>,
) -> Box<Insertion> {
    return Box::new(Insertion {
        table_name: table_name.to_string(),
        column_names: column_names.map(|column_names| {
            column_names
                .iter()
                .map(|column_name| column_name.clone())
                .collect()
        }),
        values: Box::new(values.iter().map(|v| v.clone()).collect()),
    });
}

impl Insertion {
    pub fn validate(&self) -> Result<(), String> {
        return self
            .column_names
            .as_ref()
            .map(|column_names| {
                if self.values.len() != column_names.len() {
                    return Err(format!(
                        "{} values for {} columns",
                        self.values.len(),
                        column_names.len()
                    ));
                }

                Ok(())
            })
            .map_or_else(|| Ok(()), |r| r);
    }

    pub fn column_names(&self) -> Option<Iter<String>> {
        self.column_names
            .as_ref()
            .and_then(|column_names| Some(column_names.iter()))
    }

    pub fn values(&self) -> Iter<Value> {
        self.values.iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Null,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite3;

    #[test]
    fn parses_insertion_statement() {
        let statement = "INSERT INTO apples(slices) VALUES(15);";
        let parse_result = sqlite3::AstParser::new().parse(statement);
        if parse_result.is_err() {
            parse_result.expect("should parse insertion statement");
        } else {
            let insert_stmt = parse_result.unwrap();
            assert_eq!(
                insert_stmt,
                Ast::Insert(new_insertion(
                    "apples",
                    Some(vec!["slices".to_string()]),
                    vec![Value::Integer(15)],
                ))
            )
        }
    }

    #[test]
    fn parses_create_table_statement() {
        let statement = "CREATE TABLE apples(slices INTEGER);";
        let parse_result = sqlite3::AstParser::new().parse(statement);
        if parse_result.is_err() {
            parse_result.expect("should parse create table statement");
        } else {
            let insert_stmt = parse_result.unwrap();
            assert_eq!(
                insert_stmt,
                Ast::Create(TableSchema {
                    name: "apples".to_string(),
                    columns: vec![Column {
                        name: Some("slices".to_string()),
                        datatype: Datatype::Integer,
                    }]
                })
            )
        }
    }
}
