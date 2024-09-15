use std::{
    env,
    error::Error,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use crate::instruction::{Instruction, StdoutTo};
use crate::text_processing;

pub struct Session {
    cwd: PathBuf,                //Current working directory
    pub exit_code: String,       //Status of last executed program
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
        print!("..{}", parent_dir);
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
    Also manages the exit code.
    */
    pub fn execute_input(&mut self) -> Result<(), Box<dyn Error>> {
        let instructions = text_processing::parse_input(&self.input)?;
        //Holds stdout from command run in previous iteration, as bytes
        let mut pipe = Vec::new();

        for (i, instruction) in instructions.iter().enumerate() {
            let program = instruction.command[0].as_str();

            if self.builtins.contains(&program) {
                self.run_builtin(&instruction, &mut pipe)?;
            } else {
                let mut process_builder = Command::new(program);
                process_builder.args(&instruction.command[1..]);
                //The process being executed with this iteration
                let mut current_process: Child;

                let mut stdio_handle = Stdio::inherit();
                match instruction.stdout_to {
                    StdoutTo::Stdout => {
                        //Send stdout of process to stdout
                        stdio_handle = Stdio::inherit();
                    }
                    StdoutTo::Pipe => {
                        //Send stdout of process to pipe for next command
                        stdio_handle = Stdio::piped();
                    }
                    _ => {}
                }
                process_builder.stdout(stdio_handle);

                //If this command follows a pipe
                if i > 0 {
                    process_builder.stdin(Stdio::piped());
                }
                if let Ok(child) = process_builder.spawn() {
                    current_process = child;
                } else {
                    self.exit_code = "?".to_string();
                    return Err(Box::from(format!("Command '{}' not found.", program)));
                }
                //Some(), if the stdin of current_process is being captured.
                //i.e., this only executes if i > 0 (=command follows a pipe)
                if let Some(mut child_stdin) = current_process.stdin.take() {
                    //Write stdout from previous command to stdin of this process
                    child_stdin.write(&pipe)?;
                    pipe.clear();
                }
                //Some(), if stdout of current_process is being captured
                if let Some(mut child_stdout) = current_process.stdout.take() {
                    child_stdout.read_to_end(&mut pipe)?;
                }

                //Wait for process to finish and collect exit status
                if let Ok(exit_status) = current_process.wait() {
                    if let Some(code) = exit_status.code() {
                        self.exit_code = code.to_string();
                    } else {
                        //No exit code (process was terminated by a signal)
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
    fn run_builtin(
        &mut self,
        instruction: &Instruction,
        pipe: &mut impl Write,
    ) -> Result<(), Box<dyn Error>> {
        match instruction.command[0].as_str() {
            "cd" => {
                if let Some(target_path) = instruction.command.get(1) {
                    env::set_current_dir(Path::new(target_path))?;
                } else {
                    //Change to home directory
                }
                self.cwd = env::current_dir()?;
            }
            "pwd" => {
                let result = format!("{}", self.cwd.display().to_string());
                if let StdoutTo::Pipe = instruction.stdout_to {
                    write!(pipe, "{}\n", result)?;
                } else {
                    println!("{}", result);
                }
            }
            "pushd" => {
                if let Some(target_path) = instruction.command.get(1) {
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
                    let msg = "Please specify a directory".to_string();
                    return Err(Box::from(msg));
                }
            }
            "popd" => {
                if self.dir_stack.len() == 0 {
                    let msg = "Directory stack empty.".to_string();
                    return Err(Box::from(msg));
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
