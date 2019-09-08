extern crate pest;
#[macro_use]
extern crate pest_derive;

use crate::pest::Parser;

#[derive(Parser)]
#[grammar = "sqlite3.pest"]
pub struct Sqlite3Parser;

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt");
    'main: loop {
        let readline = rl.readline("sqlite> ");
        match readline {
            Ok(buffer) => {
                rl.add_history_entry(buffer.as_str());
                let parse_result = Sqlite3Parser::parse(Rule::program, buffer.as_str());
                if parse_result.is_err() {
                    println!("Unrecognized command \'{}\'.", buffer);
                    continue;
                }
                let ast = parse_result.unwrap();
                for program in ast {
                    let statements = program.into_inner();
                    for statement in statements {
                        match statement.as_rule() {
                            Rule::meta_command => {
                                let meta_commands = statement.into_inner();
                                for meta_command in meta_commands {
                                    match meta_command.as_rule() {
                                        Rule::exit => {
                                            break 'main;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Rule::EOI => {}
                            _ => println!("Unsupported command \'{}\'.", buffer),
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
