use snafu::prelude::*;
use serde::Serialize;

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};


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
    #[snafu(display("Entry `{}` in collection `{}` symlinks to a seemingly missing directory:\n{}", path.display(), name, source))]
    BrokenEntrySymlink { name: String, path: PathBuf, source: std::io::Error },
}

#[derive(Debug)]
// TODO: Database needs to be an RwLock wrapper around the actual type... so we can actually reload it from routes every TTL seconds
/// A Database is loaded from disk. The database succeeds even when the actual disk operations failed, so that a message
/// can be presented to the end-user on the web interface. The Database has a cache with a configurable TTL (in seconds):
///   - when loading from disk fails, the cache is disabled until it succeeds
///   - when loading succeeds, the cache is refreshed every TTL seconds
pub struct Database {
    /// Where the database files are stored on disk
    pub basedir: PathBuf,
    /// The number of seconds to wait before refreshing the database entries
    pub ttl: usize,
    /// The collections found in the database, hidden behind an Arc<RwLock>> to enable read-write multithreading
    /// The entries are stored as an Option<Vec<Collection>> which is none as long as the loading fails
    pub collections: Arc<RwLock<Option<Vec<Collection>>>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Collection {
    pub name: String,
    pub path: PathBuf,
}

impl Database {
    pub fn from_dir(basedir: &Path, errors: &mut Vec<String>) -> Database {
        let mut database = Database {
            basedir: basedir.to_path_buf(),
            ttl: 60, // TODO
            collections: Arc::new(RwLock::new(None)),
        };
        database.load(basedir, errors);
        database
    }

    pub fn load(&mut self, basedir: &Path, errors: &mut Vec<String>) {
        match basedir.canonicalize().context(NoBasedirSnafu { path: basedir.to_path_buf() }) {
            Ok(p) => {
                self.basedir = p;
                match self.load_collections() {
                    Ok(c) => {
                        let mut store = self.collections.write().unwrap();
                        *store = Some(c);
                        // TODO: if there are other top-level errors than database error (for example config errors)
                        // need to stop overwriting them
                        *errors = Vec::new();
                    }, Err(e) => {
                        errors.push(e.to_string().replace("\n", "<br>"));
                    }
                }
            }, Err(e) => {
                errors.push(e.to_string().replace("\n", "<br>"));
            }
        }
    }

    pub fn load_collections(&self) -> Result<Vec<Collection>, DatabaseError> {
        let mut collections: Vec<Collection> = Vec::new();
        for entry in std::fs::read_dir(&self.basedir).context(NoBasedirSnafu { path: &self.basedir })? {
            let entry = entry.context(FailedAnonymousFileSnafu { path: &self.basedir })?;
            let name = entry.file_name().into_string().unwrap();
            let path = entry.path().canonicalize().context(BrokenEntrySymlinkSnafu { name: &name, path: entry.path() })?;
            let collection = Collection { name, path };
            collections.push(collection);
        }
        Ok(collections)
    }

    pub fn collections(&self) -> Vec<Collection> {
        //let collections: Vec<Collection> = self.collections.read().unwrap().unwrap().as_ref().clone();
        if let Some(c) = &*self.collections.read().unwrap() {
            c.clone()
        } else {
            Vec::new()
        }
    }

    pub fn has_collection(&self, name: &str) -> bool {
        self.collections().iter().find(|x| x.name == name).is_some()
    }
}
