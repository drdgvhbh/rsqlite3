use rustyline::error::ReadlineError;
use rustyline::Editor;

mod ast;
mod sqlite3;

use ast::Ast;

fn main() {
    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt").ok();
    'main: loop {
        let readline = rl.readline("sqlite> ");
        match readline {
            Ok(buffer) => {
                rl.add_history_entry(buffer.as_str());
                let parse_result = sqlite3::AstParser::new().parse(buffer.as_str());
                if parse_result.is_err() {
                    println!("Unrecognized command \'{}\'.", buffer);
                    continue;
                }
                let ast = parse_result.ok().unwrap();
                match ast {
                    Ast::Exit => break 'main,
                    Ast::Create(t) => println!("{:#?}", t),
                    Ast::Insert(i) => println!("{:#?}", i),
                }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Ast, Column, Datatype, Insertion, Table, Value},
        sqlite3,
    };

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
                Ast::Create(Table {
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
