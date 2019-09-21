use self::database::data::serializer::MessagePackSerializer;
use self::database::{Column, DataType, Schema};
use database::{factory::Factory, TableValue};
use std::fs::{create_dir_all, File};
use std::sync::Mutex;

mod database;
use rustyline::error::ReadlineError;
use rustyline::Editor;
mod ast;
mod sql;
use lalrpop_util::ParseError;

use clap;
use clap::{App, Arg, SubCommand};

pub use self::ast::{Ast, MetaCommand, SQLStatement};

fn main() {
    let matches = App::new("SQLearn")
        .version("1.0")
        .author("Ryan Lee. <ryanleecode@gmail.com>")
        .about("Simple REPL SQL Database")
        .arg(
            Arg::with_name("db_dir")
                .short("d")
                .long("dir")
                .value_name("DIR")
                .help("Sets the database directory")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("page_size")
                .short("p")
                .long("page-size")
                .value_name("PAGE_SIZE")
                .help("Sets the page size in bytes")
                .default_value("64")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("table_file_ext")
                .short("e")
                .long("ext")
                .value_name("TABLE_FILE_EXT")
                .help("Sets file extension for tables")
                .default_value("table")
                .takes_value(true),
        )
        .get_matches();

    let db_dir = matches.value_of("db_dir").unwrap();
    let page_size_str = matches.value_of("page_size").unwrap();
    if page_size_str.parse::<usize>().is_err() {
        println!(
            "Error: expected page_size to be a positive integer but found {}",
            page_size_str
        );
        return;
    }
    let page_size: usize = page_size_str.parse().unwrap();
    let table_file_ext = matches.value_of("table_file_ext").unwrap();

    create_dir_all(db_dir).expect("should be able to create database directory");
    let factory_conf = database::factory::FactoryConfiguration {
        database_dir: db_dir.into(),
        table_file_ext: table_file_ext.into(),
        serializer: MessagePackSerializer {},
        page_byte_size: page_size,
    };
    let factory = Mutex::new(Factory::new(factory_conf));
    let db = database::Database::new(factory);
    /*
    db.insert(
        "asdf",
        vec![TableValue::Int(13), TableValue::Char("yolo".to_string())],
    )
    .unwrap();
    */

    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt").ok();
    let print_err = |err: &str| println!("Error: {}", err.to_string());
    'main: loop {
        let readline = rl.readline("db> ");
        match readline {
            Ok(buffer) => {
                rl.add_history_entry(buffer.as_str());
                let parse_result = sql::AstParser::new().parse(buffer.as_str());
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
                    Ast::MetaCommand(meta_command) => match meta_command {
                        MetaCommand::Exit => break 'main,
                    },
                    Ast::SQLStatement(sql_statement) => match sql_statement {
                        SQLStatement::Create(table_schema) => {
                            let result = db.create_table(table_schema);
                            if result.is_err() {
                                print_err(&result.unwrap_err());
                            }
                        }
                        SQLStatement::Insert(insertion) => {
                            let result = db.insert(&insertion.table_name, insertion.values);
                            if result.is_err() {
                                print_err(&result.unwrap_err());
                            }
                        }
                        SQLStatement::Select(selection) => {
                            /*                          let result = executor.select(selection);
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
                            } */
                        }
                    },
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
