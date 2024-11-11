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
    let mut session = Session::new();
    let mut result;
    let mut input_control = Input::new(&mut stdout);
    let mut input: String;
    loop {
        result = input_control.prompt(&session.exit_code, &session.cwd);
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

        if let Err(msg) = session.execute_input(&input, false) {
            Output::shell_error(input_control.stdout, msg);
        } else {
            session.exit_code = "0".to_string();
        }
        input_control.input.clear();
    }
}
