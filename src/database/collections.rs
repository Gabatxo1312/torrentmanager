use serde::Serialize;
use snafu::prelude::*;

use std::path::{Path, PathBuf};

use crate::database::*;
use crate::utils::read_dir::*;

#[derive(Clone, Debug)]
pub struct CollectionDB {
    collections: Vec<Collection>,
}

impl CollectionDB {
    pub fn list(&self) -> &[Collection] {
        &self.collections
    }

    pub fn has(&self, name: &str) -> bool {
        self.list().iter().any(|x| x.name == name)
    }

    pub fn get(&self, name: &str) -> Option<&Collection> {
        self.list().iter().find(|x| x.name == name)
    }

    pub fn load(collections_dir: &Path) -> Result<Self, DatabaseError> {
        let collections = read_dir_into(collections_dir).context(ReadDirSnafu)?;

        Ok(Self { collections })
    }
}

#[derive(Clone, Debug, Serialize)]
/// A Collection is a folder (or symlink to a folder) in the collections dir.
///
/// We make sure the folder is writable!
pub struct Collection {
    pub name: String,
    pub path: PathBuf,
}

impl TryFrom<ReadDirEntry> for Collection {
    type Error = ReadDirError;

    fn try_from(entry: ReadDirEntry) -> Result<Self, Self::Error> {
        ensure_writable_dir(&entry.path)
            .boxed()
            .context(TransformSnafu)?;

        Ok(Collection {
            name: entry.name,
            // The path was canonicalized by ensure_writable_dir so this should not fail
            path: entry.path.canonicalize().unwrap(),
        })
    }
}

impl Collection {
    /// Get the top-level folders in a collection
    pub fn folders(&self) -> Result<Vec<String>, DatabaseError> {
        Ok(read_dir(&self.path)
            .context(ReadDirSnafu)?
            .into_iter()
            .map(|entry| entry.name)
            .collect())
    }
}
