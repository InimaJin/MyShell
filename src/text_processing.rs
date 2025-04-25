use std::error::Error;

use crate::{
    instruction::{Instruction, StdoutTo},
    utils,
};

/*
Parses the user's input and returns a vector holding one or more Instruction(s).
Multiple commands are separated by pipes as the user enters their input,
resulting in multiple Instructions.
*/
pub fn parse_input(input: &str) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let mut all_instructions: Vec<Instruction> = Vec::new();

    let mut instruction = Instruction::new();

    let mut quote_opened = false;
    let mut quote_type = ' ';
    //Keep track of the number of subcommands open in current iteration
    let mut subs_opened = 0;
    //The temporary elements being built and eventually pushed into instruction.command
    let mut current_element = String::new();
    let mut temp_string = String::new();
    //Whether or not temp_string should actually be pushed into current_element after each iteration
    let mut push_allowed: bool;
    let chars: Vec<char> = input.chars().collect();
    for (i, c_ref) in chars.iter().enumerate() {
        if !instruction.read_from_pipe {
            if let Some(prev_instruction) = all_instructions.last() {
                if let StdoutTo::Pipe = prev_instruction.stdout_to {
                    instruction.read_from_pipe = true
                }
            }
        }
        push_allowed = true;
        let c = *c_ref;
        if c == '"' || c == '\'' {
            push_allowed = false;
            if !quote_opened {
                quote_type = c;
                quote_opened = true;
            } else if quote_opened && c == quote_type {
                quote_opened = false;
            } else {
                push_allowed = true;
            }
        } else if !quote_opened {
            temp_string.clear();
            if c.is_whitespace() && subs_opened == 0 {
                push_allowed = false;
                if current_element.len() != 0 {
                    if let StdoutTo::File(_) = instruction.stdout_to {
                        instruction.filename = current_element;
                    } else {
                        instruction.command.push(current_element);
                    }
                    current_element = String::new();
                }
            } else if c == '|' && subs_opened == 0 {
                push_allowed = false;
                if let StdoutTo::Stdout = instruction.stdout_to {
                    instruction.stdout_to = StdoutTo::Pipe;
                }
                if current_element.len() != 0 {
                    if let StdoutTo::File(_) = instruction.stdout_to {
                        instruction.filename = current_element;
                    } else {
                        instruction.command.push(current_element);
                    }
                    current_element = String::new();
                }
                if instruction.command.len() != 0 {
                    all_instructions.push(instruction);
                    instruction = Instruction::new();
                }
            } else if c == '~' {
                let home_pathbuf = utils::home_dir()?;
                temp_string = format!("{}", home_pathbuf.display());
            } else if c == '>' {
                push_allowed = false;
                if let StdoutTo::File(_) = instruction.stdout_to {
                    continue;
                }
                //Overwrite file
                let mut write_mode = 'o';
                if let Some(next_char) = chars.get(i + 1) {
                    if *next_char == c {
                        //Append to file
                        write_mode = 'a';
                    }
                }
                instruction.stdout_to = StdoutTo::File(write_mode);
            } else if c == '$' {
                if let Some('{') = chars.get(i + 1) {
                    subs_opened += 1;
                }
                if subs_opened > 1 {
                    temp_string.push_str("$");
                } else {
                    push_allowed = false;
                }
            } else if subs_opened > 0 {
                if c == '{' {
                    if subs_opened > 1 {
                        temp_string.push_str("{");
                    } else {
                        push_allowed = false;
                    }
                } else if c == '}' {
                    subs_opened -= 1;
                    if subs_opened == 0 {
                        push_allowed = false;
                        instruction.command.push(current_element);
                        instruction
                            .subcommand_indices
                            .push(instruction.command.len() - 1);

                        current_element = String::new();
                    } else {
                        temp_string.push_str("}");
                    }
                }
            }
        }

        if push_allowed {
            if temp_string.is_empty() {
                //If none of the above conditions were met, simply
                //push the character
                current_element.push(c);
            } else {
                current_element.push_str(&temp_string);
            }
            if i == input.len() - 1 {
                if let StdoutTo::File(_) = instruction.stdout_to {
                    instruction.filename = current_element.clone();
                } else {
                    instruction.command.push(current_element.clone());
                }
                current_element.clear();
            }
        }
    }

    if !current_element.is_empty() {
        instruction.command.push(current_element);
    }

    //The last command in user's input is followed by whitespace and needs
    //to be added here.
    if instruction.command.len() != 0 {
        all_instructions.push(instruction);
    }

    Ok(all_instructions)
}
