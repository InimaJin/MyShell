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
    let mut all_instructions = Vec::new();

    let mut instruction = Instruction::new();

    let mut quote_opened = false;
    let mut quote_type = ' ';
    //Keep track of the number of subcommands open in current iteration
    let mut subs_opened = 0;
    let mut current_element = String::new();
    let chars: Vec<char> = input.chars().collect();
    for (i, c_ref) in chars.iter().enumerate() {
        let c = *c_ref;
        if c == '"' || c == '\'' {
            if !quote_opened {
                quote_type = c;
                quote_opened = true;
            } else if quote_opened && c == quote_type {
                quote_opened = false;
            }
        } else if c.is_whitespace() && !quote_opened && subs_opened == 0 {
            if current_element.len() != 0 {
                if let StdoutTo::File(_) = instruction.stdout_to {
                    instruction.filename = current_element;
                } else {
                    instruction.command.push(current_element);
                }
                current_element = String::new();
            }
        } else if c == '|' && !quote_opened && subs_opened == 0 {
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
        } else if c == '~' && !quote_opened {
            let home_pathbuf = utils::home_dir()?;
            let home_string = format!("{}", home_pathbuf.display());
            current_element.push_str(home_string.as_str());
        } else if c == '>' && !quote_opened {
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
        } else if c == '$' && !quote_opened {
            if i < chars.len() - 1 && chars[i + 1] == '{' {
                subs_opened += 1;
            }
            if subs_opened > 1 {
                current_element.push_str(c.to_string().as_str());
            }
        } else if subs_opened > 0 && !quote_opened {
            if c == '{' {
                if subs_opened > 1 {
                    current_element.push_str(c.to_string().as_str());
                }
            } else if c == '}' {
                subs_opened -= 1;
                if subs_opened == 0 {
                    instruction.command.push(current_element);
                    instruction
                        .subcommand_indices
                        .push(instruction.command.len() - 1);

                    current_element = String::new();
                } else {
                    current_element.push_str(c.to_string().as_str());
                }
            } else {
                current_element.push_str(c.to_string().as_str());
            }
        } else {
            current_element.push_str(c.to_string().as_str());
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
