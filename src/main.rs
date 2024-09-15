mod instruction;
mod session;
mod text_processing;

use session::Session;

fn main() {
    let mut session = Session::build();

    loop {
        if let Err(e) = session.prompt_for_input() {
            eprintln!("ERROR: {}", e);
            break;
        };
        if session.input.trim() == "exit" {
            println!("Goodbye.");
            break;
        } else if session.input.trim().len() == 0 {
            session.exit_code = 0.to_string();
            continue;
        }

        if let Err(msg) = session.execute_input() {
            eprintln!("Shell error:\n{}", msg);
        };
    }
}
