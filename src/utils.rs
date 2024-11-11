use home;
use std::{error::Error, fs, path::PathBuf};

pub fn home_dir() -> Result<PathBuf, String> {
    if let Some(pathbuf) = home::home_dir() {
        return Ok(pathbuf);
    } else {
        let msg = "Failed to retrieve home directory.".to_string();
        return Err(msg);
    }
}

pub fn config_dir() -> Result<PathBuf, Box<dyn Error>> {
    let mut config_dir = PathBuf::new();
    if let Ok(home_dir) = home_dir() {
        config_dir = home_dir;
    }

    for element in [".config", "myshell"] {
        config_dir.push(element);
    }
    if !config_dir.exists() {
        fs::create_dir(&config_dir)?;
    }

    Ok(config_dir)
}

pub fn write_history(input: &str) -> Result<(), Box<dyn Error>> {
    if input.len() == 0 {
        return Ok(());
    }
    if let Ok(mut histfile_path) = config_dir() {
        histfile_path.push("history");
        let mut history: Vec<u8> = Vec::new();
        if histfile_path.exists() {
            history = read_history()?;
        }
        for b in format!("{}\n", input).as_bytes() {
            history.push(*b);
        }
        fs::write(&histfile_path, history)?;
    }

    Ok(())
}

/* 
Returns the contents of the history file as bytes.
*/
pub fn read_history() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut histfile_path = config_dir()?;
    histfile_path.push("history");
    let history = fs::read(histfile_path)?;
    Ok(history)
}
