//Information on what to do with stdout of process:
//Send to stdout/ pipe to stdin of subsequent process/ write to a file
pub enum StdoutTo {
    Stdout,
    Pipe,
    File(String),
}

pub struct Instruction {
    pub command: Vec<String>, //Command to execute
    pub stdout_to: StdoutTo,  //See enum StdoutTo
}
impl Instruction {
    pub fn new() -> Self {
        Self {
            command: Vec::new(),
            stdout_to: StdoutTo::Stdout,
        }
    }
}
