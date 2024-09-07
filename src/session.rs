use std::{
    env,
    error::Error,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

mod text_processing;

pub struct Session {
    cwd: PathBuf,
    pub exit_code: String,
    pub input: String,
    builtins: Vec<&'static str>,
}

impl Session {
    pub fn build() -> Self {
        Session {
            cwd: env::current_dir().unwrap_or(PathBuf::new()),
            exit_code: String::from("0"),
            input: String::new(),
            builtins: vec!["cd", "pwd"],
        }
    }

    /*
    Prompts user for input and
    assigns that data to self.input
    */
    pub fn prompt_for_input(&mut self) -> Result<(), Box<dyn Error>> {
        if self.exit_code != 0.to_string() {
            print!("|{}|", self.exit_code);
        }
        print!("{}>> ", self.cwd.display());
        io::stdout().flush()?;

        self.input.clear();
        io::stdin().read_line(&mut self.input)?;

        Ok(())
    }

    /*
    Input is parsed by parse_input() from text_processing.
    Executes the command(s). If multiple commands are specified
    using pipes, they are executed in order and their stdout
    is always redirected to the subsequent command.
    Also manages the exit code
    */
    pub fn execute_input(&mut self) -> Result<(), String> {
        let commands_to_run = text_processing::parse_input(&self.input);

        //Process that was run in previous iteration of loop
        let mut previous_process: Option<std::process::Child> = None;
        for (i, command) in commands_to_run.iter().enumerate() {
            let program = command[0].as_str();

            if self.builtins.contains(&program) {
                self.run_builtin(&command);
            } else {
                let mut process_builder = Command::new(program);
                process_builder.args(&command[1..]);
                let mut current_process: std::process::Child;
                let mut stdio_handle: Stdio;
                if i == 0 { //If this is the first command in commands_to_run
                    if commands_to_run.len() > 1 { //If there are more commands following
                        stdio_handle = Stdio::piped();
                    } else {
                        stdio_handle = Stdio::inherit();
                    }
                    current_process = process_builder.stdout(stdio_handle).spawn().unwrap();
                } else {  //If this is the last command in commands_to_run
                    if i == commands_to_run.len() - 1 {
                        stdio_handle = Stdio::inherit();
                    } else {
                        stdio_handle = Stdio::piped();
                    }
                    current_process = process_builder
                    .stdout(stdio_handle)
                    .stdin(Stdio::from(previous_process.unwrap().stdout.unwrap()))
                    .spawn()
                    .unwrap();
                } 

                let exit_status_result = current_process.wait();
                previous_process = Some(current_process);

                if let Ok(exit_status) = exit_status_result {
                    if let Some(code) = exit_status.code() {
                        self.exit_code = code.to_string();
                    } else {
                        // No exit code (process was terminated by a signal)
                        self.exit_code = "!".to_string();
                    }
                } else {
                    self.exit_code = "?".to_string();
                }
            }
        }

        Ok(())
    }

    /*
    Determines which builtin command has been issued
    and runs the appropriate logic
    */
    fn run_builtin(&mut self, command: &[String]) {
        match command[0].as_str() {
            "cd" => {
                if let Some(target_path) = command.get(1) {
                    env::set_current_dir(Path::new(target_path));
                } else {
                    // Change to home directory
                }
                self.cwd = env::current_dir().expect("Failed to read current working directory.")
            }
            "pwd" => {
                println!("{}", self.cwd.display());
            }
            _ => {}
        }
    }
}
