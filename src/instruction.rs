//Specifies how to handle stdout of process
#[derive(Debug)]
pub enum StdoutTo {
    Stdout,     //Send to stdout
    Pipe,       //pipe to stdin of subsequent process
    File(char), //Write to file (mode indicated by char; o = overwrite, a = append)
}

//Template for building a single process later on
#[derive(Debug)]
pub struct Instruction {
    pub command: Vec<String>,           //Command to execute
    pub stdout_to: StdoutTo,            //See enum StdoutTo
    pub filename: String,               //Empty if not writing stdout to file
    pub subcommand_indices: Vec<usize>, //Indices of subcommands within this command
}
impl Instruction {
    pub fn new() -> Self {
        Self {
            command: Vec::new(),
            stdout_to: StdoutTo::Stdout,
            filename: String::new(),
            subcommand_indices: Vec::new(),
        }
    }
}
