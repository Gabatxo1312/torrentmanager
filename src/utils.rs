use std::env::{split_paths, var_os};
use std::path::{Path, PathBuf};

// https://stackoverflow.com/questions/37498864/finding-executable-in-path-with-rust
pub fn find_it<P>(exe_name: P) -> bool
where
    P: AsRef<Path>,
{
    var_os("PATH")
        .and_then(|paths| {
            split_paths(&paths)
                .filter_map(|dir| {
                    let full_path = dir.join(&exe_name);
                    if full_path.is_file() {
                        Some(full_path)
                    } else {
                        None
                    }
                })
                .next()
        })
        .is_some()
}

pub fn xdg_config_file<P>(file: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    if let Some(cfgdir) = var_os("XDG_CONFIG_HOME") {
        let path = PathBuf::from(cfgdir);
        Some(path.join("TorrentManager").join(file))
    } else if let Some(homedir) = var_os("HOME") {
        let path = PathBuf::from(homedir);
        Some(path.join(".config/TorrentManager").join(file))
    } else {
        None
    }
}

pub fn unwrap_or_err<T, E>(result: Result<T, E>, errors: &mut Vec<E>) -> Option<T> {
    match result {
        Ok(r) => Some(r),
        Err(e) => {
            errors.push(e);
            None
        }
    }
}
