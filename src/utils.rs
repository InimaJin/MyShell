use home;
use std::path::PathBuf;

pub fn home_dir() -> Result<PathBuf, String> {
    if let Some(pathbuf) = home::home_dir() {
        return Ok(pathbuf);
    } else {
        let msg = "Failed to retrieve home directory.".to_string();
        return Err(msg);
    }
}
