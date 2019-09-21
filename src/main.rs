use self::database::data::serializer::MessagePackSerializer;
use self::database::{Column, DataType, Schema};
use database::{factory::Factory, TableValue};
use std::cmp::PartialOrd;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::{self};
use std::fs::{create_dir_all, File};
use std::sync::Mutex;

mod database;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs::OpenOptions;
mod ast;
mod sql;
use lalrpop_util::ParseError;

use clap;
use clap::{App, Arg, SubCommand};
use database::table;

pub use self::ast::{Ast, MetaCommand, SQLStatement};

struct CommandLineArguments {
    db_dir: String,
    page_size: usize,
    table_file_ext: String,
}

fn main() {
    execute_program().unwrap();
}

fn execute_program() -> Result<(), String> {
    let print_err = |err: &str| println!("Error: {}", err.to_string());
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

    let cli_args = parse_command_line_arguments(&matches)?;
    let db_dir = cli_args.db_dir.clone();
    let table_file_ext = cli_args.table_file_ext.clone();
    let page_size = cli_args.page_size;

    create_dir_all(&db_dir).map_err(|_| format!("should be able to create database directory"))?;
    let factory_conf = database::factory::FactoryConfiguration {
        database_dir: db_dir.clone(),
        table_file_ext: table_file_ext.clone(),
        serializer: MessagePackSerializer {},
        page_byte_size: page_size,
    };
    let factory = Factory::new(factory_conf);

    let mut tables = HashMap::new();
    let table_files = load_database_table_files(&db_dir, &table_file_ext)?;
    for table_file in table_files {
        let table = factory.load_table_from_file(table_file)?;
        tables.insert(table.name().to_string(), table);
    }

    let mut db = database::Database::new(Mutex::new(factory), tables);

    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt").ok();
    'main: loop {
        let readline = rl.readline("db> ");
        match readline {
            Ok(buffer) => {
                rl.add_history_entry(buffer.as_str());
                let parse_result = sql::AstParser::new().parse(buffer.as_str());
                if parse_result.is_err() {
                    print_parse_error(&parse_result.unwrap_err());
                    continue;
                }
                let ast = parse_result.ok().unwrap();
                match ast {
                    Ast::MetaCommand(meta_command) => match meta_command {
                        MetaCommand::Exit => {
                            let result = db.flush();
                            if result.is_err() {
                                print_err(&result.unwrap_err());
                            }
                            break 'main;
                        }
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
                            let result = db.select(&selection.table_name, &selection.columns);
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
                    },
                }
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
            _ => {
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
    Ok(())
}

fn load_database_table_files(db_dir: &str, table_file_ext: &str) -> Result<Vec<File>, String> {
    let mut files = vec![];
    for entry in fs::read_dir(db_dir).map_err(|err| format!("{}", err))? {
        let entry = entry.map_err(|err| format!("{}", err))?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap() == table_file_ext {
            files.push(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(path)
                    .map_err(|err| format!("{}", err))?,
            );
        }
    }

    Ok(files)
}

fn parse_command_line_arguments(
    matches: &clap::ArgMatches,
) -> Result<CommandLineArguments, String> {
    let db_dir = matches.value_of("db_dir").unwrap();
    let page_size_str = matches.value_of("page_size").unwrap();
    if page_size_str.parse::<usize>().is_err() {
        return Err(format!(
            "Error: expected page_size to be a positive integer but found {}",
            page_size_str
        ));
    }
    let page_size: usize = page_size_str.parse().unwrap();
    let table_file_ext = matches.value_of("table_file_ext").unwrap();

    Ok(CommandLineArguments {
        db_dir: db_dir.into(),
        page_size,
        table_file_ext: table_file_ext.into(),
    })
}

fn print_parse_error<T: Display + Debug, E: Debug>(parse_error: &ParseError<usize, T, E>) {
    match parse_error {
        ParseError::UnrecognizedToken { token, expected: _ } => {
            println!("Unexpected token \"{}\" at column {}.", token.1, token.0)
        }
        ParseError::UnrecognizedEOF {
            location,
            expected: _,
        } => {
            if location > &0 {
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
}
