use crossterm::{
    cursor::{MoveLeft, MoveRight},
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Colors, Print, SetAttribute, SetColors},
    terminal::{self, Clear, ClearType},
};
use std::{
    error::Error,
    io::{Stdout, Write},
    path::PathBuf,
    time::Duration,
};

pub struct Input;
pub struct Output;

impl Input {
    pub fn prompt(
        stdout: &mut Stdout,
        exit_code: &str,
        cwd: &PathBuf,
    ) -> Result<String, Box<dyn Error>> {
        let mut prompt = String::from("\r");
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
        prompt.push_str("> ");
        execute!(
            stdout,
            //SetAttribute(Attribute::Bold),
            SetColors(Colors {
                foreground: Some(Color::White),
                background: None
            }),
            Print(prompt),
            SetAttribute(Attribute::Reset)
        )?;

        terminal::enable_raw_mode()?;
        let mut input = Vec::new();
        let mut input_cursor = 0;
        let mut ev: Event;
        loop {
            if event::poll(Duration::from_millis(100))? {
                ev = event::read()?;
                match ev {
                    Event::Key(key_ev) => {
                        match key_ev.code {
                            KeyCode::Char(ch) => {
                                if key_ev.modifiers == KeyModifiers::CONTROL && ch == 'c' {
                                    return Err(Box::from("Quit".to_string()));
                                } else {
                                    if input_cursor < input.len() {
                                        Self::replace_char(&mut input, input_cursor, ch, stdout)?;
                                    } else {
                                        //Character has to be appended
                                        input.push(ch);
                                        execute!(stdout, Print(ch))?;
                                    }
                                    input_cursor += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let finished_input =
                                    input.iter().map(|c| c.to_string()).collect::<String>();
                                terminal::disable_raw_mode()?;
                                execute!(stdout, Print("\r\n"))?;
                                return Ok(finished_input);
                            }
                            KeyCode::Left => {
                                if input_cursor > 0 {
                                    input_cursor -= 1;
                                    execute!(stdout, MoveLeft(1))?;
                                }
                            }
                            KeyCode::Right => {
                                if input_cursor < input.len() {
                                    input_cursor += 1;
                                    execute!(stdout, MoveRight(1))?;
                                }
                            }
                            KeyCode::Backspace => {
                                if input_cursor > 0 {
                                    queue!(
                                        stdout,
                                        MoveLeft(input_cursor as u16),
                                        Clear(ClearType::UntilNewLine),
                                    )?;
                                    if input_cursor < input.len() {
                                        input.remove(input_cursor - 1);
                                        queue!(
                                            stdout,
                                            Print(
                                                input
                                                    .iter()
                                                    .map(|c| c.to_string())
                                                    .collect::<String>()
                                            ),
                                            MoveLeft((input.len() - input_cursor + 1) as u16)
                                        )?;
                                    } else {
                                        input.pop();
                                        queue!(
                                            stdout,
                                            Print(
                                                input
                                                    .iter()
                                                    .map(|c| c.to_string())
                                                    .collect::<String>()
                                            )
                                        )?;
                                    }
                                    stdout.flush()?;
                                    input_cursor -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /*
    "Injects" a character into the user's input
    and renders the updated string to the screen, overwriting
    what was displayed before
    */
    fn replace_char(
        input: &mut Vec<char>,
        input_cursor: usize,
        ch: char,
        stdout: &mut Stdout,
    ) -> Result<(), Box<dyn Error>> {
        input.insert(input_cursor, ch);
        if input_cursor > 0 {
            queue!(stdout, MoveLeft(input_cursor as u16))?;
        }
        queue!(
            stdout,
            Print(input.iter().map(|c| c.to_string()).collect::<String>()),
            MoveLeft((input.len() - input_cursor - 1) as u16)
        )?;
        stdout.flush()?;

        Ok(())
    }
}

impl Output {
    pub fn shell_error(stdout: &mut Stdout, err: Box<dyn Error>) {
        execute!(
            stdout,
            Print(format!(
                "{}{}Shell error:{}\n{}\n",
                SetColors(Colors {
                    foreground: Some(Color::DarkRed),
                    background: None
                }),
                SetAttribute(Attribute::Bold),
                SetAttribute(Attribute::Reset),
                err
            ))
        );
    }
}
