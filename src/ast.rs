use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum Ast {
    Exit,
    Create(TableSchema),
    Insert(Insertion),
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
    pub column_names: Option<Vec<String>>,
    pub values: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
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
                Ast::Insert(Insertion {
                    table_name: "apples".to_string(),
                    column_names: Some(vec!["slices".to_string()]),
                    values: vec![Value::Integer(15)],
                })
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
