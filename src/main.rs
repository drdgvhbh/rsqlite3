use exitcode;

use std::io::Write;

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
        if buffer == ".exit" {
            std::process::exit(exitcode::OK)
        } else {
            println!("Unrecognized command \'{}\'.", buffer);
        }
    }
}
