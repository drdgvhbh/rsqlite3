#![feature(trait_alias)]

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod ast;
mod bptree;
mod executor;
mod pager;
mod sqlite3;
mod table;

use bptree::BPTree;
use lalrpop_util::ParseError;

use ast::{Ast, Value};

fn main() {
    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt").ok();
    let mut executor = executor::Executor::new();
    let bptree_degree = 4;
    let print_err = |err: &str| println!("Error: {}", err.to_string());
    'main: loop {
        let readline = rl.readline("sqlite> ");
        match readline {
            Ok(buffer) => {
                rl.add_history_entry(buffer.as_str());
                let parse_result = sqlite3::AstParser::new().parse(buffer.as_str());
                if parse_result.is_err() {
                    match parse_result.unwrap_err() {
                        ParseError::UnrecognizedToken { token, expected: _ } => {
                            println!("Unexpected token \"{}\" at column {}.", token.1, token.0)
                        }
                        ParseError::UnrecognizedEOF {
                            location,
                            expected: _,
                        } => {
                            if location > 0 {
                                println!("Unexpected EOF at column {}", location);
                            }
                        }
                        ParseError::InvalidToken { location } => {
                            println!("Invalid token at column {}", location);
                        }
                        ParseError::ExtraToken { token } => {
                            println!("Extra token \"{}\" at column {}", token.1, token.0)
                        }
                        err => {
                            println!("{:#?}", err);
                        }
                    }
                    continue;
                }
                let ast = parse_result.ok().unwrap();
                match ast {
                    Ast::Exit => break 'main,
                    Ast::Create(schema) => {
                        let result = table::Table::new(
                            schema,
                            BPTree::<Value, Vec<Value>>::new(bptree_degree),
                        );
                        match result {
                            Err(err) => print_err(&err),
                            Ok(table) => {
                                let result = executor.add_table(table);
                                if result.is_err() {
                                    print_err(&result.unwrap_err());
                                }
                            }
                        }
                    }
                    Ast::Insert(insertion) => {
                        let result = executor.insert(insertion);
                        if result.is_err() {
                            print_err(&result.unwrap_err());
                        }
                    }
                    Ast::Select(selection) => {
                        let result = executor.select(selection);
                        match result {
                            Err(err) => print_err(&err),
                            Ok(rows) => {
                                for row in rows {
                                    for val in &row[..row.len() - 1] {
                                        print!("{}|", val);
                                    }
                                    for val in &row[(row.len() - 1)..] {
                                        print!("{}\n", val);
                                    }
                                }
                            }
                        }
                    }
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
