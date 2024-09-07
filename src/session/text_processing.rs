/*
Parses the user's input.
The vector returned by this function holds one or more vectors,
each representing a command from within the user's input. Multiple
commands are separated by pipes as the user enters their input.
*/
pub fn parse_input(input: &str) -> Vec<Vec<String>> {
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
                subcommand.push(current_element.clone());
                current_element.clear();
            }
        } else if c == '|' && !quote_opened {
            if current_element.len() != 0 {
                subcommand.push(current_element.clone());
                current_element.clear();
            }
            result.push(subcommand.clone());
            subcommand.clear()
        } else {
            current_element.push_str(c.to_string().as_str());
        }
    }
    result.push(subcommand.clone());
    result
}
