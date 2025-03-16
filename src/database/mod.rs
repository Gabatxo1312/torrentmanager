use snafu::prelude::*;

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::AppState;

mod save_paths;
pub use save_paths::SavePathDB;
mod uploads;
pub use uploads::{UploadDB, UploadID, UploadPath};
mod collections;
pub use collections::{CollectionDB, Collection};

#[derive(Debug, Snafu)]
pub enum DatabaseError {
    #[snafu(display("Could not read collections basedir `{}`:\n{}", path.display(), source))]
    NoBasedir { path: PathBuf, source: std::io::Error },
    #[snafu(display("Failed to read collection entry `{}`:\n{}", path.display(), source))]
    FailedEntry { path: PathBuf, source: std::io::Error },
    #[snafu(display("Failed to read file `{}`:\n{}", path.display(), source))]
    FailedAnonymousFile { path: PathBuf, source: std::io::Error },
    #[snafu(display("Collection `{}` symlinks to a seemingly missing directory `{}`:\n{}", name, path.display(), source))]
    BrokenCollectionSymlink { name: String, path: PathBuf, source: std::io::Error },
    #[snafu(display("Failed to read collections directory {}:\n{source}", path.display()))]
    FailedReadCollection { path: PathBuf, source: std::io::Error },
    #[snafu(display("Failed to read collection entry in collection {}:\n{source}", collection.display()))]
    FailedReadCollectionEntry { collection: PathBuf, source: std::io::Error },
    #[snafu(display("Entry contains invalid UTF-8 characters: {}", osstring.to_string_lossy()))]
    FailedUnicode { osstring: std::ffi::OsString },
    #[snafu(display("Entry `{}` in collection `{}` symlinks to a seemingly missing directory:\n{}", path.display(), name, source))]
    BrokenEntrySymlink { name: String, path: PathBuf, source: std::io::Error },
    #[snafu(display("Could not find upload dir {}:\n{source}", path.display()))]
    NoUploadDir { path: PathBuf, source: std::io::Error },
    #[snafu(display("Upload dir is not a folder: {}))", path.display()))]
    NotUploadDir { path: PathBuf },
    #[snafu(display("Failed to read upload dir:\n{source}"))]
    ReadUploadDir { source: std::io::Error },
    #[snafu(display("Failed to read upload dir entry {} at path {}:\n{}", id.to_string(), path.display(), source))]
    ReadUploadDirEntry { id: UploadID, path: PathBuf, source: std::io::Error },
    #[snafu(display("Failed to write upload entry {path}:\n{source}"))]
    WriteUploadDirEntry { path: String, source: std::io::Error },
    #[snafu(display("Invalid content id {id}"))]
    InvalidContentID { id: String },
    #[snafu(display("Could not find torrents_dir: {}", path.display()))]
    NoTorrentDir { path: PathBuf, source: std::io::Error },
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
    /// Where the database files are stored on disk
    pub basedir: PathBuf,
    /// The number of seconds to wait before refreshing the database entries
    pub ttl: usize,
    /// The collections found in the database, hidden behind an Arc<RwLock>> to enable read-write multithreading
    /// The entries are stored as an Option<Vec<Collection>> which is none as long as the loading fails
    //pub collections: Arc<RwLock<Vec<Collection>>>,
    pub upload_dir: PathBuf,
    pub torrents_dir: PathBuf,
    // TEMPORARY: place in Cache
    pub collections: Vec<Collection>,
}


impl Database {
    pub fn from_dirs(basedir: &Path, upload_dir: &Path, torrents_dir: &Path) -> Result<Database, DatabaseError> {
        // Check that folders exist
        let basedir = basedir.canonicalize().context(NoBasedirSnafu { path: basedir.to_path_buf() })?;
        let upload_dir = upload_dir.canonicalize().context(NoUploadDirSnafu { path: upload_dir.to_path_buf() })?;
        let torrents_dir = torrents_dir.canonicalize().context(NoTorrentDirSnafu { path: torrents_dir.to_path_buf() })?;
        if upload_dir.is_dir() {
            Ok(Database {
                collections: CollectionDB::load(&basedir)?,
                basedir,
                ttl: 60, // TODO
                upload_dir,
                // TODO: missing torrents dir error
                torrents_dir: torrents_dir.to_path_buf(),
            })
        } else {
            Err(DatabaseError::NotUploadDir { path: upload_dir.to_path_buf() })
        }
    }

    pub fn uploads(&self) -> UploadDB {
        UploadDB::from(&self.upload_dir)
    }

    pub fn save_paths(&self) -> SavePathDB {
        SavePathDB::from(&self.torrents_dir)
    }

    pub fn collections(&self) -> CollectionDB {
        CollectionDB::with(&self.collections)
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
