use std::path::{Path, PathBuf};

// Path to ~/.local/share/torrentmanager, or PWD/torrentmanager
pub fn data_dir() -> PathBuf {
    dirs::data_dir().unwrap_or_default().join("torrentmanager")
}

// Path to ~/.config/torrentmanager/{file} or PWD/{file}
pub fn config_file<P>(file: P) -> PathBuf
where
    P: AsRef<Path>,
{
    dirs::config_dir().unwrap_or_default().join(file)
}
