use std::{
    error::Error,
    io::{self, Stdout, Write},
    path::PathBuf,
    time::Duration,
};

use crossterm::{
    cursor::{MoveLeft, MoveRight},
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

use crate::utils;

/*
For managing user input aspects, such as
arrow key functionality (cursor movement, history navigation) etc.
*/
pub struct Input {
    pub stdout: Stdout,   //For writing to stdout
    pub input: Vec<char>, //Vector holding user's input, updated in real time
    input_cursor: usize,  //x-location of terminal cursor relative to prompt (leftmost is 0)
}
pub struct Output;

impl Input {
    pub fn new() -> Self {
        Self {
            stdout: io::stdout(),
            input: Vec::new(),
            input_cursor: 0,
        }
    }

    /*
    Displays the prompt and calls read_keys() to get the user's input, finally returns it as a String.
    */
    pub fn prompt(&mut self, exit_code: &str, cwd: &PathBuf) -> Result<String, Box<dyn Error>> {
        let mut prompt = String::new();
        //Trying to fetch the last component of cwd
        if let Some(os_str) = cwd.file_name() {
            if let Some(str_slice) = os_str.to_str() {
                prompt.push_str(&format!("..{}", str_slice));
            }
        }
        if prompt.is_empty() {
            //E.g. if cwd is the "/" dir
            prompt.push_str(&cwd.display().to_string());
        }
        let mut prompt_color = Color::White;
        let text_color = prompt_color;
        if exit_code != "0" {
            prompt_color = Color::DarkRed;
        }
        prompt.push_str("> ");
        execute!(
            self.stdout,
            SetAttribute(Attribute::Bold),
            SetForegroundColor(prompt_color),
            Print(prompt),
            SetForegroundColor(text_color), //If prompt is read, user input should still be white
            SetAttribute(Attribute::NormalIntensity)
        )?;

        terminal::enable_raw_mode()?;
        self.input_cursor = 0;

        return Ok(self.read_keys()?);
    }

    /*
    Reads the user's keyboard input key by key and returns a string
    representing the characters.
    */
    pub fn read_keys(&mut self) -> Result<String, Box<dyn Error>> {
        //For navigating through history file using the
        //arrow up/down keys
        let mut history_pointer: Option<usize> = None;
        loop {
            if event::poll(Duration::from_millis(100))? {
                let ev = event::read()?;
                if let Event::Key(key_ev) = ev {
                    match key_ev.code {
                        KeyCode::Char(ch) => {
                            if key_ev.modifiers == KeyModifiers::CONTROL && ch == 'c' {
                                //TODO
                            } else {
                                if self.input_cursor < self.input.len() {
                                    self.insert_char(ch)?;
                                } else {
                                    //Character has to be appended
                                    self.input.push(ch);
                                    execute!(self.stdout, Print(ch))?;
                                }
                                self.input_cursor += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let finished_input =
                                self.input.iter().map(|c| c.to_string()).collect::<String>();
                            execute!(self.stdout, ResetColor, Print("\r\n"))?;
                            utils::write_history(&finished_input)?;
                            terminal::disable_raw_mode()?;
                            return Ok(finished_input);
                        }
                        KeyCode::Left => {
                            if self.input_cursor > 0 {
                                self.input_cursor -= 1;
                                execute!(self.stdout, MoveLeft(1))?;
                            }
                        }
                        KeyCode::Right => {
                            if self.input_cursor < self.input.len() {
                                self.input_cursor += 1;
                                execute!(self.stdout, MoveRight(1))?;
                            }
                        }
                        KeyCode::Backspace => {
                            if self.input_cursor > 0 {
                                queue!(
                                    self.stdout,
                                    MoveLeft(self.input_cursor as u16),
                                    Clear(ClearType::UntilNewLine),
                                )?;
                                if self.input_cursor < self.input.len() {
                                    self.input.remove(self.input_cursor - 1);
                                    queue!(
                                        self.stdout,
                                        Print(
                                            self.input
                                                .iter()
                                                .map(|c| c.to_string())
                                                .collect::<String>()
                                        ),
                                        MoveLeft((self.input.len() - self.input_cursor + 1) as u16)
                                    )?;
                                } else {
                                    self.input.pop();
                                    queue!(
                                        self.stdout,
                                        Print(
                                            self.input
                                                .iter()
                                                .map(|c| c.to_string())
                                                .collect::<String>()
                                        )
                                    )?;
                                }
                                self.stdout.flush()?;
                                self.input_cursor -= 1;
                            }
                        }
                        //Navigating through history
                        KeyCode::Up | KeyCode::Down => {
                            //true => "Up" key was pressed. false => "Down" key pressed
                            let up = if let KeyCode::Up = key_ev.code {
                                true
                            } else {
                                false
                            };
                            let history = String::from_utf8(utils::read_history()?)?;
                            let lines = history.lines().collect::<Vec<&str>>();
                            if let Some(val) = history_pointer {
                                if up && val != 0 {
                                    history_pointer = Some(val - 1);
                                } else if !up && val < lines.len() - 1 {
                                    history_pointer = Some(val + 1);
                                }
                            } else {
                                history_pointer = Some(lines.len() - 1);
                            }
                            let next_command = lines[history_pointer.unwrap()];

                            if !self.input.is_empty() {
                                self.clear_prompt()?;
                            }
                            //Write the new input into the prompt
                            execute!(self.stdout, Print(next_command))?;

                            self.input = next_command.chars().collect();
                            self.input_cursor = self.input.len();
                        }
                        KeyCode::Tab => {
                            println!("{:?}", utils::bin_dir_contents());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /*
    Clears everything currently written to prompt by user,
    leaving the prompt itself in place
    */
    fn clear_prompt(&mut self) -> Result<(), Box<dyn Error>> {
        if self.input_cursor != 0 {
            queue!(self.stdout, MoveLeft(self.input_cursor as u16))?;
        }
        queue!(self.stdout, Clear(ClearType::UntilNewLine))?;

        self.stdout.flush()?;
        Ok(())
    }

    /*
    "Injects" a character into the user's input
    and renders the updated string to the screen, overwriting
    what was displayed before
    */
    fn insert_char(&mut self, ch: char) -> Result<(), Box<dyn Error>> {
        self.input.insert(self.input_cursor, ch);
        if self.input_cursor > 0 {
            queue!(self.stdout, MoveLeft(self.input_cursor as u16))?;
        }
        queue!(
            self.stdout,
            Print(self.input.iter().map(|c| c.to_string()).collect::<String>()),
            MoveLeft((self.input.len() - self.input_cursor - 1) as u16)
        )?;
        self.stdout.flush()?;

        Ok(())
    }
}

impl Output {
    pub fn shell_error(stdout: &mut Stdout, err: Box<dyn Error>) {
        execute!(
            stdout,
            Print(format!(
                "{}{}Shell error:{}\n{}\n",
                SetForegroundColor(Color::DarkRed),
                SetAttribute(Attribute::Bold),
                SetAttribute(Attribute::Reset),
                err
            ))
        )
        .unwrap();
    }
}
