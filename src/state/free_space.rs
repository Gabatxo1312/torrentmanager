use human_bytes::human_bytes;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::ReloadableState;

#[derive(Clone)]
pub struct FreeSpace {
    pub path: PathBuf,
    pub bytes: f64,
}

impl FreeSpace {
    pub fn new<T: AsRef<Path>>(path: T) -> FreeSpace {
        let dir = path.as_ref().to_path_buf();
        let bytes: f64 = if let Ok(output) = std::process::Command::new("df")
            .arg("--output=avail")
            .arg(&dir)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let size = stdout.lines().next_back().unwrap().trim();
            f64::from_str(size).unwrap() * 1024.0
        } else {
            eprintln!("Failed to load partition size for {}!", dir.display());
            0.0
        };

        FreeSpace { path: dir, bytes }
    }
}

impl ReloadableState for FreeSpace {
    fn reload_state(&mut self) {
        let new_space = FreeSpace::new(&self.path);
        self.bytes = new_space.bytes;
    }
}

impl std::fmt::Display for FreeSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", human_bytes(self.bytes))
    }
}
