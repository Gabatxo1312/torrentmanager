pub use qbt::utils::magnet_hash;
pub use qbt::utils::torrent_hash;

use std::env::{var_os, split_paths};
use std::path::Path;

// https://stackoverflow.com/questions/37498864/finding-executable-in-path-with-rust
pub fn find_it<P>(exe_name: P) -> bool
    where P: AsRef<Path>,
{
    var_os("PATH").and_then(|paths| {
        split_paths(&paths).filter_map(|dir| {
            let full_path = dir.join(&exe_name);
            if full_path.is_file() {
                Some(full_path)
            } else {
                None
            }
        }).next()
    }).is_some()
}
