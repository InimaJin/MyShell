use home;
use std::{
    error::Error, fs, os::unix::fs::PermissionsExt, path::PathBuf
};

pub fn home_dir() -> Result<PathBuf, String> {
    if let Some(pathbuf) = home::home_dir() {
        return Ok(pathbuf);
    } else {
        let msg = "Failed to retrieve home directory.".to_string();
        return Err(msg);
    }
}

/*
Creates the config directory (~/.config/myshell) if nonexistent and returns the path
*/
pub fn config_dir() -> Result<PathBuf, Box<dyn Error>> {
    let mut config_dir: PathBuf;
    config_dir = home_dir()?;

    for element in [".config", "myshell"] {
        config_dir.push(element);
    }
    if !config_dir.exists() {
        fs::create_dir(&config_dir)?;
    }

    Ok(config_dir)
}

/*
Writes the user's input to history file located at the path <config_dir>/history
*/
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

pub fn bin_dir_contents() -> Result<Vec<String>, Box<dyn Error>> {
    let mut contents = Vec::new();
    if let Ok(read_dir) = fs::read_dir("/bin/") {
        contents = read_dir
            .map(|result| {
                if let Ok(dir_entry) = result {
                    dir_entry.path()
                } else {
                    PathBuf::new()
                }
            }).filter(|pathbuf| {
                if let Ok(metadata) = pathbuf.metadata() {
                    metadata.permissions().mode() & 0o111 != 0
                } else {
                    false
                }
            }).map(|pathbuf| {
                let mut filename_string = String::new();
                if let Some(os_str) = pathbuf.file_name() {
                    if let Some(str_slice) = os_str.to_str() {
                        filename_string = str_slice.to_string();
                    }
                }
                
                filename_string
            })
            .collect();
    }
        
    Ok(contents)
}