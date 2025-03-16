use snafu::prelude::*;
use serde::Serialize;

use std::path::{Path, PathBuf};

use crate::database::*;

#[derive(Clone)]
pub struct CollectionDB<'a> {
    collections: &'a [Collection],
}

impl <'a> CollectionDB<'a> {
    pub fn with(collections: &'a [Collection]) -> CollectionDB<'a> {
        CollectionDB {
            collections,
        }
    }

    pub fn list(&'a self) -> &'a [Collection] {
        &self.collections
    }

    pub fn has(&'a self, name: &str) -> bool {
        self.list().iter().find(|x| x.name == name).is_some()
    }

    pub fn get(&'a self, name: &str) -> Option<&'a Collection> {
        self.list().iter().find(|x| x.name == name)
    }

    pub fn load(basedir: &Path) -> Result<Vec<Collection>, DatabaseError> {
        let mut collections: Vec<Collection> = Vec::new();
        for entry in std::fs::read_dir(basedir).context(NoBasedirSnafu { path: basedir.to_path_buf() })? {
            let entry = entry.context(FailedAnonymousFileSnafu { path: basedir.to_path_buf() })?;
            let name = entry.file_name().into_string().unwrap();
            let path = entry.path().canonicalize().context(BrokenEntrySymlinkSnafu { name: &name, path: entry.path() })?;
            let collection = Collection { name, path };
            collections.push(collection);
        }
        Ok(collections)
    }

}




#[derive(Clone, Debug, Serialize)]
/// A Collection is a folder in the collections_dir
pub struct Collection {
    pub name: String,
    pub path: PathBuf,
}

impl Collection {
    /// Get the top-level folders in a collection
    pub fn folders(&self) -> Result<Vec<String>, DatabaseError> {
        let mut entries: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(&self.path).context(FailedReadCollectionSnafu { path: self.path.clone() })? {
            let entry = entry.context(FailedReadCollectionEntrySnafu { collection: self.path.clone() })?;
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            } else {
                return Err(DatabaseError::FailedUnicode { osstring: entry.file_name() });
            }
        }
        Ok(entries)
    }
}
