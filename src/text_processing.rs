use home;
use std::error::Error;

use crate::instruction::{Instruction, StdoutTo};

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
    let mut current_element = String::new();
    for c in input.chars() {
        if c == '"' || c == '\'' {
            if !quote_opened {
                quote_type = c;
                quote_opened = true;
            } else if quote_opened && c == quote_type {
                quote_opened = false;
            }
        } else if c.is_whitespace() && !quote_opened {
            if current_element.len() != 0 {
                instruction.command.push(current_element);
                current_element = String::new();
            }
        } else if c == '|' && !quote_opened {
            instruction.stdout_to = StdoutTo::Pipe;
            if current_element.len() != 0 {
                instruction.command.push(current_element);
                current_element = String::new();
            }
            if instruction.command.len() != 0 {
                all_instructions.push(instruction);
                instruction = Instruction::new();
            }
        } else if c == '~' && !quote_opened {
            if let Some(pathbuf) = home::home_dir() {
                if let Some(str_slice) = pathbuf.to_str() {
                    current_element.push_str(str_slice);
                }
            } else {
                let msg = "Failed to retrieve home directory.".to_string();
                return Err(Box::from(msg));
            }
        } else {
            current_element.push_str(c.to_string().as_str());
        }
    }
    //The last command in user's input is followed by whitespace and needs
    //to be added here.
    if instruction.command.len() != 0 {
        all_instructions.push(instruction);
    }

    Ok(all_instructions)
}
