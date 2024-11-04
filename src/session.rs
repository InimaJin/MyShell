use std::{
    env,
    error::Error,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use crate::{
    instruction::{Instruction, StdoutTo},
    text_processing, utils,
};

pub struct Session {
    pub cwd: PathBuf,            //Current working directory
    pub exit_code: String,       //Status of last executed program
    builtins: Vec<&'static str>, //Shell builtin commands
    dir_stack: Vec<PathBuf>,     //For pushd/ popd
    pipe: Vec<u8>,               //Stdout from command run in previous iteration, as bytes
}

impl Session {
    pub fn new() -> Self {
        Session {
            cwd: env::current_dir().unwrap_or(PathBuf::new()),
            exit_code: String::from("0"),
            builtins: vec!["cd", "pwd", "pushd", "popd"],
            dir_stack: vec![],
            pipe: vec![],
        }
    }

    /*
    Input is parsed by parse_input() from text_processing.

    Executes the command(s). If multiple commands are specified
    using pipes, they are executed in order and their stdout
    is always redirected to the subsequent command.

    If a subcommand (e.g. ${whoami}) is being executed, it returns
    Ok(Some(Stdout_of_subcommand_as_string)), otherwise Ok(None).

    Also manages the exit code.
    */
    pub fn execute_input(
        &mut self,
        as_subcommand: bool,
        input: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let instructions = text_processing::parse_input(input)?;
        for (i, instruction) in instructions.iter().enumerate() {
            let mut command = instruction.command.clone();
            let program = command[0].clone();
            if program == "ls" {
                //command.insert(1, "--color=auto".to_string());
            }

            println!("\n\r{instruction:?}");


            for subcommand_i in instruction.subcommand_indices.iter() {
                //Execute the subcommand and store its stdout
                let output = self.execute_input(true, &command[*subcommand_i])?;
                //Substitute the subcommand with its computed stdout.
                //Unwrap() will not panic, since at this point,
                //output is Some() because as_subcommand was set to true.
                command[*subcommand_i] = output.unwrap();
            }

            if self.builtins.contains(&&program[..]) {
                self.run_builtin(&instruction, command, as_subcommand)?;
            } else {
                let mut process_builder = Command::new(&program[..]);
                process_builder.args(&command[1..]);

                let mut current_process: Child;

                let mut stdio_handle;
                match instruction.stdout_to {
                    StdoutTo::Stdout => {
                        //Send stdout of process to stdout
                        stdio_handle = Stdio::inherit();
                    }
                    StdoutTo::Pipe => {
                        //Send stdout of process to pipe for next command
                        stdio_handle = Stdio::piped();
                    }
                    StdoutTo::File(_) => {
                        stdio_handle = Stdio::piped();
                    }
                }

                if as_subcommand {
                    stdio_handle = Stdio::piped();
                }

                process_builder.stdout(stdio_handle);

                //If current iteration's command (instruction) follows a pipe
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
                    //Write stdout from previous command (now in pipe) to stdin of this process
                    child_stdin.write(&self.pipe)?;
                }
                //Some(), if stdout of current_process is being captured
                if let Some(mut child_stdout) = current_process.stdout.take() {
                    self.pipe.clear();
                    child_stdout.read_to_end(&mut self.pipe)?;
                    if let StdoutTo::File(mode) = instruction.stdout_to {
                        self.write_to_file(&instruction, mode)?;
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
                } else {
                    self.exit_code = "?".to_string();
                }
            }
        }

        if as_subcommand {
            let mut command_output = String::from_utf8(self.pipe.clone())?;
            command_output = command_output.trim().to_string();
            return Ok(Some(command_output));
        }

        Ok(None)
    }

    /*
    Determines which builtin command has been issued
    and runs the appropriate logic
    */
    fn run_builtin(
        &mut self,
        instruction: &Instruction,
        command: Vec<String>,
        as_subcommand: bool,
    ) -> Result<(), Box<dyn Error>> {
        match command[0].as_str() {
            "cd" => {
                if let Some(target_path) = command.get(1) {
                    env::set_current_dir(Path::new(target_path))?;
                } else {
                    let home_pathbuf = utils::home_dir()?;
                    env::set_current_dir(home_pathbuf)?;
                }
                self.cwd = env::current_dir()?;
            }
            "pwd" => {
                let output = format!("{}", self.cwd.display().to_string());
                if let StdoutTo::Stdout = instruction.stdout_to {
                    if !as_subcommand {
                        println!("{}", output);
                    }
                } else {
                    self.pipe.clear();
                    write!(&mut self.pipe, "{}\n", output)?;
                    if let StdoutTo::File(mode) = instruction.stdout_to {
                        self.write_to_file(instruction, mode)?;
                    }
                }
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
                    self.exit_code = "!".to_string();
                    let msg = "Please specify a directory".to_string();
                    return Err(Box::from(msg));
                }
            }
            "popd" => {
                if self.dir_stack.len() == 0 {
                    self.exit_code = "!".to_string();
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

        self.exit_code = "0".to_string();

        Ok(())
    }

    /*
    Invoked by '>' operator.
    Writes the stdout (held in the pipe) to a file.
    */
    fn write_to_file(
        &mut self,
        instruction: &Instruction,
        mode: char,
    ) -> Result<(), Box<dyn Error>> {
        if instruction.filename.is_empty() {
            let program = &instruction.command[0];
            let msg = format!("Please specify target file for output of '{}'.", program);
            return Err(Box::from(msg));
        }

        let mut data_to_write = Vec::new();
        match mode {
            'o' => {
                data_to_write.append(&mut self.pipe);
            }
            'a' => {
                data_to_write = fs::read(&instruction.filename)?;
                data_to_write.append(&mut self.pipe);
            }
            _ => {}
        }

        fs::write(&instruction.filename, &data_to_write)?;

        Ok(())
    }
}
