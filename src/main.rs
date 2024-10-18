use std::io;

mod instruction;
mod session;
mod text_processing;
mod user;
mod utils;

use session::Session;
use user::{Input, Output};

fn main() {
    let mut stdout = io::stdout();
    let mut session = Session::build();
    let mut result;
    let mut input: String;
    loop {
        result = Input::prompt(&mut stdout, &session.exit_code, &session.cwd);
        if let Err(e) = result {
            eprintln!("ERROR: {}", e);
            break;
        } else {
            input = result.unwrap();
        }
        if input.trim() == "exit" {
            println!("Goodbye.");
            break;
        } else if input.trim().len() == 0 {
            session.exit_code = 0.to_string();
            continue;
        }

        if let Err(msg) = session.execute_input(false, &input) {
            Output::shell_error(&mut stdout, msg);
        }
        input.clear();
    }
}
