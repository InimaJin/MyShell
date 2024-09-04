use my_shell::Session;

fn main() {
    let mut session = Session::build();
    
    loop {
        if let Err(e) = session.prompt_for_input() {
            eprintln!("ERROR: {}", e);
            break;
        };   
        if session.input == "exit" {
            println!("Goodbye.");
            break;
        } else if session.input.len() == 0 {
            session.exit_code = 0.to_string();
            continue;
        }
    
        session.execute_input();
    }
}