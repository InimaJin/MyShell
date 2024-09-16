//Specifies how to handle stdout of process
pub enum StdoutTo {
    Stdout,     //Send to stdout
    Pipe,       //pipe to stdin of subsequent process
    File(char), //Write to file (TO-DO: char represents write mode (overwrite/ append))
}

//Template for building a single process later on
pub struct Instruction {
    pub command: Vec<String>, //Command to execute
    pub stdout_to: StdoutTo,  //See enum StdoutTo
    pub filename: String,     // Empty if not writing stdout to file
}
impl Instruction {
    pub fn new() -> Self {
        Self {
            command: Vec::new(),
            stdout_to: StdoutTo::Stdout,
            filename: String::new(),
        }
    }
}
