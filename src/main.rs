use exitcode;

use std::io::Write;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use crate::pest::Parser;

#[derive(Parser)]
#[grammar = "sqlite3.pest"]
pub struct Sqlite3Parser;

fn print_prompt() {
    print!("db > ")
}

fn main() {
    loop {
        print_prompt();
        if std::io::stdout().flush().is_err() {
            std::process::exit(exitcode::IOERR)
        }
        let mut buffer = String::new();
        if std::io::stdin().read_line(&mut buffer).is_err() {
            std::process::exit(exitcode::IOERR)
        }
        buffer.truncate(buffer.trim_end().len());
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
                                Rule::exit => std::process::exit(exitcode::OK),
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
}
