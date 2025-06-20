use snafu::prelude::*;

use std::path::{Path, PathBuf};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ReadDirError {
    #[snafu(display("Failed to read directory entries in {}:\n{}", path.display(), source))]
    Dir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Failed to read directory entry {}:\n{}", path.display(), source))]
    DirEntry {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("{}", source))]
    /// Occurs when read_dir_into fails to TryFrom the DirEntry
    Transform {
        source: Box<dyn snafu::Error + Send + Sync + 'static>,
    },
    #[snafu(display("Unknown error at path {}: {}", path.display(), source))]
    Other {
        path: PathBuf,
        source: Box<dyn snafu::Error + Send + Sync + 'static>,
    },
}

#[derive(Clone, Debug)]
pub struct ReadDirEntry {
    pub name: String,
    pub path: PathBuf,
}

impl ReadDirEntry {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }
}

pub fn read_dir(dir: &Path) -> Result<Vec<ReadDirEntry>, ReadDirError> {
    let mut entries: Vec<ReadDirEntry> = Vec::new();
    for entry in std::fs::read_dir(dir).context(DirSnafu {
        path: dir.to_path_buf(),
    })? {
        let entry = entry.context(DirSnafu {
            path: dir.to_path_buf(),
        })?;
        let name = entry.file_name().into_string().unwrap();
        entries.push(ReadDirEntry::new(name, entry.path().to_path_buf()));
    }

    Ok(entries)
}

pub fn read_dir_into<T: TryFrom<ReadDirEntry>>(dir: &Path) -> Result<Vec<T>, ReadDirError>
where
    ReadDirError: From<<T as TryFrom<ReadDirEntry>>::Error>,
{
    let mut entries: Vec<T> = Vec::new();

    for entry in read_dir(dir)? {
        let typed_entry = T::try_from(entry)?;
        entries.push(typed_entry);
    }

    Ok(entries)
}

/// Symlinks are not followed. Only directories are recursed into.
pub fn read_dir_recursive_into<T: TryFrom<ReadDirEntry>>(dir: &Path) -> Result<Vec<T>, ReadDirError>
where
    ReadDirError: From<<T as TryFrom<ReadDirEntry>>::Error>,
{
    let mut entries: Vec<T> = Vec::new();

    for entry in read_dir(dir)? {
        if entry.path.is_dir() {
            entries.extend(read_dir_recursive_into(&entry.path)?);
        }

        let typed_entry = T::try_from(entry)?;
        entries.push(typed_entry);
    }

    Ok(entries)
}
