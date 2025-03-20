use snafu::prelude::*;

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::config::DirConfig;
use crate::utils::read_dir::ReadDirError;
use crate::utils::writable_dir::{ensure_writable_dir, WritableDirError};
use crate::AppState;

mod save_paths;
pub use save_paths::SavePathDB;
mod uploads;
pub use uploads::{UploadDB, UploadID, UploadPath};
mod collections;
pub use collections::{Collection, CollectionDB};

#[derive(Debug, Snafu)]
pub enum DatabaseError {
    #[snafu(display("Collections dir is not writable:\n{}", source))]
    Collections {
        source: WritableDirError,
    },
    #[snafu(display("Uploads dir is not writable:\n{}", source))]
    Uploads {
        source: WritableDirError,
    },
    #[snafu(display("Downloads dir is not writable:\n{}", source))]
    Downloads {
        source: WritableDirError,
    },

    #[snafu(display("{source}"))]
    ReadDir {
        source: ReadDirError,
    },
    #[snafu(display("Failed to persist upload: {source}"))]
    PersistUpload {
        source: WritableDirError,
    },

    // #[snafu(display("Failed to list directory entries in `{}`:\n{}", path.display(), source))]
    // ReadDir { path: PathBuf, source: std::io::Error },
    // #[snafu(display("Failed to read directory entry `{}`:\n{}", path.display(), source))]
    // ReadEntry { path: PathBuf, source: std::io::Error },

    // BELOW ARE OLD ERRORS TO BE REPLACED
    // #[snafu(display("Failed to read directory entry `{}`:\n{}", path.display(), source))]
    // FailedEntry {
    //     path: PathBuf,
    //     source: std::io::Error,
    // },
    // #[snafu(display("Failed to read file `{}`:\n{}", path.display(), source))]
    // FailedAnonymousFile {
    //     path: PathBuf,
    //     source: std::io::Error,
    // },
    // #[snafu(display("Failed to read collections directory {}:\n{source}", path.display()))]
    // FailedReadCollection {
    //     path: PathBuf,
    //     source: std::io::Error,
    // },
    // #[snafu(display("Failed to read collection entry in collection {}:\n{source}", collection.display()))]
    // FailedReadCollectionEntry {
    //     collection: PathBuf,
    //     source: std::io::Error,
    // },
    // #[snafu(display("Entry contains invalid UTF-8 characters: {}", osstring.to_string_lossy()))]
    // FailedUnicode {
    //     osstring: std::ffi::OsString,
    // },
    // #[snafu(display("Entry `{}` in collection `{}` symlinks to a seemingly missing directory:\n{}", path.display(), name, source))]
    // BrokenEntrySymlink {
    //     name: String,
    //     path: PathBuf,
    //     source: std::io::Error,
    // },
    // ReadUploadDir {
    //     source: std::io::Error,
    // },
    // #[snafu(display("Failed to read upload dir entry {} at path {}:\n{}", id.to_string(), path.display(), source))]
    // ReadUploadDirEntry {
    //     id: UploadID,
    //     path: PathBuf,
    //     source: std::io::Error,
    // },
    #[snafu(display("Invalid content id {id}"))]
    InvalidContentID {
        id: String,
    },
    #[snafu(display("An unknown error occurred accessing {}: {}", path.display(), source))]
    Unknown {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}

#[derive(Clone, Debug)]
// TODO: Database needs to be an RwLock wrapper around the actual type... so we can actually reload it from routes every TTL seconds
///// A Database is loaded from disk. The database succeeds even when the actual disk operations failed, so that a message
///// can be presented to the end-user on the web interface. The Database has a cache with a configurable TTL (in seconds):
/////   - when loading from disk fails, the cache is disabled until it succeeds
/////   - when loading succeeds, the cache is refreshed every TTL seconds
/// The `Database` is the storage for the entire application. It is in charge of managing:
///   - submitted torrents/magnets using [`UploadDB`] (see [`Database::uploads`])
///   - content collections managed by the mediatek using [`CollectionsDB`] (see
///   [`Database::colections`])
///   - torrent save paths as consumed by the torrent clients using [`SavePathDB`] (see
///   [`Database::save_paths`]
pub struct Database {
    dirs: DirConfig,
    /// The number of seconds to wait before refreshing the database entries
    pub ttl: usize,
    // TEMPORARY: place in Cache
    pub collections: CollectionDB,
}

impl Database {
    pub fn from_dirs(dirs: &DirConfig) -> Result<Database, DatabaseError> {
        // Check that folders exist and are writable
        ensure_writable_dir(&dirs.collections).context(CollectionsSnafu)?;
        ensure_writable_dir(&dirs.uploads).context(UploadsSnafu)?;
        ensure_writable_dir(&dirs.downloads).context(DownloadsSnafu)?;

        Ok(Database {
            dirs: dirs.clone(),
            ttl: 60,
            collections: CollectionDB::load(&dirs.collections)?,
        })
    }

    pub fn uploads(&self) -> UploadDB {
        UploadDB::from(&self.dirs.uploads)
    }

    pub fn save_paths(&self) -> SavePathDB {
        SavePathDB::from(&self.dirs.downloads)
    }

    pub fn collections(&self) -> CollectionDB {
        self.collections.clone()
    }
}

pub struct Cache {
    pub collections: Arc<RwLock<Vec<Collection>>>,
}

/// A path that is guaranteed to be valid Unicode, and can be accessed as string slice or path
/// reference.
#[derive(Clone, Debug)]
pub struct UnicodePath {
    string: String,
    path: PathBuf,
}

impl UnicodePath {
    pub fn new(s: &str) -> UnicodePath {
        UnicodePath {
            string: s.to_string(),
            path: PathBuf::from(s),
        }
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn as_str(&self) -> &str {
        &self.string
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    pub fn to_string(&self) -> String {
        self.string.to_string()
    }
}
