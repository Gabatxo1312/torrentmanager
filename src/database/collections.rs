use serde::Serialize;
use snafu::prelude::*;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::database::*;
use crate::utils::read_dir::*;

#[derive(Clone, Debug)]
pub struct CollectionDB {
    /// Basedir for the collections.
    pub basedir: PathBuf,
    /// Top-level categories in the basedir.
    pub collections: Vec<Collection>,
    /// Recursively-loaded entries in the collections.
    pub entries: HashMap<Collection, Vec<CollectionEntry>>,
}

impl CollectionDB {
    /// Load collections from a directory, following symlinks recursively.
    pub fn load(collections_dir: &Path) -> Result<Self, DatabaseError> {
        let collections: Vec<Collection> =
            read_dir_into(collections_dir).context(ReadDirSnafu {
                path: collections_dir.to_path_buf(),
            })?;
        let mut entries: HashMap<Collection, Vec<CollectionEntry>> = HashMap::new();
        for collection in &collections {
            entries.insert(
                collection.clone(),
                read_dir_recursive_into(&collection.path).context(ReadDirSnafu {
                    path: collection.path.to_path_buf(),
                })?,
            );
        }

        Ok(Self {
            basedir: collections_dir.to_path_buf(),
            collections,
            entries,
        })
    }

    pub fn reload(&mut self) -> Result<(), DatabaseError> {
        *self = Self::load(&self.basedir)?;

        Ok(())
    }

    /// Get a collection by name.
    pub fn get(&self, name: &str) -> Option<&Collection> {
        self.collections.iter().find(|x| x.name == name)
    }
}

impl std::ops::Deref for CollectionDB {
    type Target = Vec<Collection>;

    fn deref(&self) -> &Self::Target {
        &self.collections
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize)]
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
            .context(ReadDirSnafu {
                path: self.path.to_path_buf(),
            })?
            .into_iter()
            .map(|entry| entry.name)
            .collect())
    }
}

/// An entry in a collection. Can be a symlink, a simple file, or a folder.
#[derive(Clone, Debug, Serialize)]
pub enum CollectionEntry {
    File(PathBuf),
    Folder(PathBuf),
    Symlink { source: PathBuf, dest: PathBuf },
}

impl TryFrom<ReadDirEntry> for CollectionEntry {
    type Error = ReadDirError;

    fn try_from(value: ReadDirEntry) -> Result<CollectionEntry, Self::Error> {
        if value.path.is_dir() {
            Ok(CollectionEntry::Folder(value.path))
        } else if value.path.is_file() {
            Ok(CollectionEntry::File(value.path))
        } else if value.path.is_symlink() {
            Ok(CollectionEntry::Symlink {
                source: value.path.to_path_buf(),
                // TODO: support reading links recursively?
                // canonicalize errors when the final dest does not exist so we don't want that
                dest: value.path.read_link().boxed().context(OtherSnafu {
                    path: value.path.to_path_buf(),
                })?,
            })
        } else {
            panic!(
                "Weird path (not file/folder/symlink): {}",
                value.path.display()
            );
        }
    }
}
