use std::path::PathBuf;

pub mod free_space;

/// Global application state.
///
/// Used to perform queries against the system, and torrentmanager's
/// database. Can be safely cloned between threads (inner mutability).
#[derive(Clone, Debug)]
pub struct AppState;

impl AppState {
    pub fn free_space(&self) -> free_space::FreeSpace {
        // TODO: configurable paths
        free_space::FreeSpace::from_path(&PathBuf::from("/home"))
    }
}
