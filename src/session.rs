use std::{
    env,
    error::Error,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

mod text_processing;

pub struct Session {
    cwd: PathBuf,                //Current working directory
    pub exit_code: String,       //Status of last exeuted program
    pub input: String,           //Input user has entered
    builtins: Vec<&'static str>, //Shell builtin commands
    dir_stack: Vec<PathBuf>,
}

impl Session {
    pub fn build() -> Self {
        Session {
            cwd: env::current_dir().unwrap_or(PathBuf::new()),
            exit_code: String::from("0"),
            input: String::new(),
            builtins: vec!["cd", "pwd", "pushd", "popd"],
            dir_stack: vec![],
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
        let mut parent_dir = String::new();
        if let Some(os_str) = self.cwd.file_name() {
            if let Some(str_slice) = os_str.to_str() {
                parent_dir.push_str(&format!("{}", str_slice));
            }
        }
        if parent_dir.len() == 0 {
            parent_dir.push_str(&self.cwd.display().to_string());
        }
        parent_dir.push_str("> ");
        print!("{}", parent_dir);
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
                let stdio_handle: Stdio;
                //If this is the first command in commands_to_run
                if i == 0 {
                    //If there are more commands following
                    if commands_to_run.len() > 1 {
                        stdio_handle = Stdio::piped();
                    } else {
                        stdio_handle = Stdio::inherit();
                    }
                    process_builder.stdout(stdio_handle);
                    if let Ok(child) = process_builder.spawn() {
                        current_process = child;
                    } else {
                        self.exit_code = "?".to_string();
                        return Err(format!("Command '{}' not found.", program));
                    }
                } else {
                    //If this is the last command in commands_to_run
                    if i == commands_to_run.len() - 1 {
                        stdio_handle = Stdio::inherit();
                    } else {
                        stdio_handle = Stdio::piped();
                    }
                    process_builder.stdout(stdio_handle).stdin(Stdio::from(
                        previous_process.take().unwrap().stdout.unwrap(),
                    ));
                    if let Ok(child) = process_builder.spawn() {
                        current_process = child;
                    } else {
                        self.exit_code = "?".to_string();
                        return Err(format!("Command '{}' not found.", program));
                    }
                }

                //Wait for process to finish and collect exit status
                if let Ok(exit_status) = current_process.wait() {
                    if let Some(code) = exit_status.code() {
                        self.exit_code = code.to_string();
                    } else {
                        //No exit code (process was terminated by a signal)
                        self.exit_code = "!".to_string();
                    }
                    previous_process = Some(current_process);
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
    fn run_builtin(&mut self, command: &[String]) -> Result<(), Box<dyn Error>> {
        match command[0].as_str() {
            "cd" => {
                if let Some(target_path) = command.get(1) {
                    env::set_current_dir(Path::new(target_path))?;
                } else {
                    //Change to home directory
                }
                self.cwd = env::current_dir()?;
            }
            "pwd" => {
                println!("{}", self.cwd.display());
            }
            "pushd" => {
                if let Some(target_path) = command.get(1) {
                    //If dir stack is empty, the current working directory
                    //becomes its first element
                    if self.dir_stack.len() == 0 {
                        self.dir_stack.push(self.cwd.clone());
                    }
                    let pathbuf = PathBuf::from(target_path);
                    env::set_current_dir(&pathbuf)?;
                    self.cwd = env::current_dir()?;
                    self.dir_stack.push(self.cwd.clone());
                } else {
                    println!("No directory specified.");
                }
            }
            "popd" => {
                if self.dir_stack.len() == 0 {
                    println!("Directory stack empty.");
                }
                //Length >= 2 because of pushd's logic
                else {
                    //Remove the last pushed directory
                    self.dir_stack.pop();
                    let len = self.dir_stack.len();
                    env::set_current_dir(&self.dir_stack[len - 1])?;
                    self.cwd = env::current_dir()?;

                    //If we've reached the directory from where pushd was
                    //called the first time, clear the stack
                    if self.dir_stack.len() == 1 {
                        self.dir_stack.pop();
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}
