use std::env::{split_paths, var_os};
use std::path::Path;

pub mod read_dir;
pub mod writable_dir;
pub mod xdg;

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

pub fn unwrap_or_err<T, E>(result: Result<T, E>, errors: &mut Vec<E>) -> Option<T> {
    match result {
        Ok(r) => Some(r),
        Err(e) => {
            errors.push(e);
            None
        }
    }
}
