use std::{
    path::PathBuf,
    env,
    error::Error,
    io::{self, Write},
    process::{Command, Stdio}
};

use shellwords;

pub struct Session {
    cwd: PathBuf,
    pub exit_code: String,
    pub input: String,
    builtins: Vec<String>
}

impl Session {
    pub fn build() -> Self {
        Session {
            cwd: env::current_dir().unwrap_or(PathBuf::new()),
            exit_code: String::from("0"),
            input: String::new(),
            builtins: vec!["cd".to_string()]
        }
    }
    
    /*
    Prompts user for input
    Assigns that data to self.input
    */
    pub fn prompt_for_input(&mut self) -> Result<(), Box<dyn Error>> {
        if self.exit_code != 0.to_string() {
            print!("|{}|", self.exit_code);
        }
        print!("{}>> ", self.cwd.display());
        io::stdout().flush().expect("Error when trying to flush stdout.");

        self.input.clear();
        io::stdin()
            .read_line(&mut self.input)?;

        self.input = self.input.trim().to_string();
        Ok(())
    }

    pub fn execute_input(&mut self) {
        let input_parsed: Vec<String> = shellwords::split(&self.input).unwrap();
        let program = &input_parsed[0];
        
        if self.builtins.contains(program) {
            println!("BUILTIN!");
        }

        let mut command = Command::new(program);
        command.args(&input_parsed[1..])
            .stdout(Stdio::inherit()) // Output of process will be displayed
            .stdin(Stdio::inherit()) // Process can read from stdin
            .stderr(Stdio::inherit()); 
        
        // Run the command
        let output = command.output().expect("Couldn't run program");
        if let Some(code) = output.status.code() {
            self.exit_code = code.to_string();
        } else { // No exit code (process was terminated by a signal)
            self.exit_code = "!".to_string();
        }
    }
}