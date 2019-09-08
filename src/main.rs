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
