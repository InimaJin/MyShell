use std::{
    env,
    error::Error,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command},
};

use crate::{
    instruction::{Instruction, StdoutTo},
    text_processing, utils,
};

use os_pipe::{self, PipeWriter};

pub struct Session {
    pub cwd: PathBuf,        //Current working directory
    pub exit_code: String,   //Status of last executed program
    dir_stack: Vec<PathBuf>, //For pushd/ popd
}

impl Session {
    const BUILTINS: [&'static str; 5] = ["cd", "pwd", "pushd", "popd", "history"];
    pub fn new() -> Self {
        Session {
            cwd: env::current_dir().unwrap_or(PathBuf::new()),
            exit_code: String::from("0"),
            dir_stack: vec![],
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
        input: &str,
        as_subcommand: bool,
    ) -> Result<Option<String>, Box<dyn Error>> {
        //Reading end of a pipe, if piping is used. Must be in outer scope because:
        //Writer process creates the pipe. The reading end will be connected to the stdin
        //of the succeeding process, so pipe_reader must survive until the next iteration.
        let mut pipe_reader = None;
        let mut instructions = text_processing::parse_input(input)?;
        let instructions_count = instructions.len();
        for (instruction_index, instruction) in instructions.iter_mut().enumerate() {
            let program = instruction.command[0].clone();
            if program == "ls" {
                instruction.command.insert(1, "--color=auto".to_string());
                //Since we have inserted something at index 1, all the subcommand indices are now 1 below what they should be
                for subcommand_i in instruction.subcommand_indices.iter_mut() {
                    *subcommand_i += 1;
                }
            }

            for &subcommand_i in instruction.subcommand_indices.iter() {
                //Execute the subcommand and store its stdout
                let output = self.execute_input(&instruction.command[subcommand_i], true)?;
                //Substitute the subcommand with its computed stdout.
                //Unwrap() will not panic, since at this point,
                //output is Some() because as_subcommand was set to true.
                instruction.command[subcommand_i] = output.unwrap();
            }

            if Self::BUILTINS.contains(&&program[..]) {
                let mut pipe_writer = None;
                if let StdoutTo::Pipe = instruction.stdout_to {
                    let (pr, pw) = os_pipe::pipe()?;
                    pipe_reader = Some(pr);
                    pipe_writer = Some(pw);
                }
                self.run_builtin(instruction, pipe_writer, as_subcommand)?;
            } else {
                let mut process_builder = Command::new(&program[..]);
                process_builder.args(&instruction.command[1..]);

                //If instruction follows a pipe, connect stdin of process to pipe created by
                //instruction from previous iteration.
                if instruction.read_from_pipe {
                    process_builder.stdin(pipe_reader.take().unwrap());
                }
                
                let mut set_up_pipe = as_subcommand;
                match instruction.stdout_to {
                    StdoutTo::Stdout => {}
                    StdoutTo::Pipe => {
                        set_up_pipe = true;
                    }
                    StdoutTo::File(_) => {}
                }
                
                if set_up_pipe {
                    if let Ok((reader, writer)) = os_pipe::pipe() {
                        //pipe_reader needs to be accessed by succeeding instruction in pipe chain.
                        pipe_reader = Some(reader);
                        //PipeWriter only needed here.
                        process_builder.stdout(writer);
                    }
                }
                
                let mut current_process: Child;
                if let Ok(child) = process_builder.spawn() {
                    current_process = child;
                } else {
                    self.exit_code = "?".to_string();
                    return Err(Box::from(format!("Command '{}' not found.", program)));
                }

                if instruction_index == instructions_count - 1 {
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
        }

        if as_subcommand {
            let mut command_output = String::new();
            if let Some(mut pr) = pipe_reader {
                pr.read_to_string(&mut command_output)?;
            }
            return Ok(Some(String::from(command_output.trim())));
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
        pipe_writer: Option<PipeWriter>, //Some(), if this instruction precedes a pipe.
        as_subcommand: bool,
    ) -> Result<(), Box<dyn Error>> {
        let mut output = String::new();
        match instruction.command[0].as_str() {
            "cd" => {
                if let Some(target_path) = instruction.command.get(1) {
                    env::set_current_dir(Path::new(target_path))?;
                } else {
                    env::set_current_dir(utils::home_dir()?)?;
                }
                self.cwd = env::current_dir()?;
            }
            "pwd" => {
                output = format!("{}", self.cwd.display().to_string());
            }
            "pushd" => {
                if let Some(target_path) = instruction.command.get(1) {
                    //If dir stack is empty, the current working directory
                    //becomes its first element
                    if self.dir_stack.is_empty() {
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
            "history" => {
                let history_result = utils::read_history()?;
                let history_string = String::from_utf8(history_result)?;
                for (i, line) in history_string.lines().enumerate() {
                    output.push_str(&format!("{} {}", i, line));
                }
            }
            _ => {}
        }

        if !output.is_empty() {
            if let Some(mut pw) = pipe_writer {
                pw.write(format!("{}\r\n", output).as_bytes())?;
            } else {
                println!("{}\r", output);
                return Ok(());
            }

            /*TODO
            if let StdoutTo::File(mode) = instruction.stdout_to {
                self.write_to_file(instruction, mode)?;
            }
             */
        }

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
                //data_to_write.append(&mut self.pipe);
            }
            'a' => {
                data_to_write = fs::read(&instruction.filename)?;
                //data_to_write.append(&mut self.pipe);
            }
            _ => {}
        }

        fs::write(&instruction.filename, &data_to_write)?;

        Ok(())
    }
}
