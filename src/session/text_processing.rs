use std::error::Error;
use home;

/*
Parses the user's input.
The vector returned by this function holds one or more vectors,
each representing a command ('subcommand') from within the user's input. Multiple
commands are separated by pipes as the user enters their input.
*/
pub fn parse_input(input: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let mut result = Vec::new();

    let mut quote_opened = false;
    let mut quote_type = ' ';
    let mut current_element = String::new();
    let mut subcommand = Vec::new();
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
                subcommand.push(current_element);
                current_element = String::new();
            }
        } else if c == '|' && !quote_opened {
            if current_element.len() != 0 {
                subcommand.push(current_element);
                current_element = String::new();
            }
            if subcommand.len() != 0 {
                result.push(subcommand);
                subcommand = Vec::new();
            }
        } else if c == '~' && !quote_opened {
            if let Some(pathbuf) = home::home_dir() {
                if let Some(str_slice) = pathbuf.to_str() {
                    current_element.push_str(str_slice);
                }
            } else {
                let msg = "Failed to retrieve home directory.".to_string();
                return Err( Box::from(msg) );
            }
        } 
        else {
            current_element.push_str(c.to_string().as_str());
        }
    }
    if subcommand.len() != 0 {
        result.push(subcommand);
    }
    Ok(result)
}
