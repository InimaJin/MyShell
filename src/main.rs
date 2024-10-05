mod instruction;
mod session;
mod text_processing;
mod utils;

use session::Session;
use termion::style;

fn main() {
    let mut session = Session::build();
    let mut input = String::new();

    loop {
        if let Err(e) = session.prompt_for_input(&mut input) {
            eprintln!("ERROR: {}", e);
            break;
        };
        if input.trim() == "exit" {
            println!("Goodbye.");
            break;
        } else if input.trim().len() == 0 {
            session.exit_code = 0.to_string();
            continue;
        }

        if let Err(msg) = session.execute_input(false, &input) {
            eprintln!(
                "{}Shell error:{}\n{}",
                style::Bold,
                style::Reset,
                msg
            );
        };
        input = String::new();
    }
}
