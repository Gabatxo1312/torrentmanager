use snafu::prelude::*;

use std::path::{Path, PathBuf};

#[derive(Debug, Snafu)]
pub enum WritableDirError {
    #[snafu(display("Directory {} does not exist", path.display()))]
    Missing { path: PathBuf },
    #[snafu(display("Symlink {} to a directory that does not exist, ir not a directory, or is not writable: {}", path.display(), source))]
    Symlink {
        path: PathBuf,
        source: Box<dyn snafu::Error + Send + Sync + 'static>,
    },
    #[snafu(display("Write permission denied to directory {}", path.display()))]
    Write { path: PathBuf },
    #[snafu(display("Not a directory or symlink: {}", path.display()))]
    Type { path: PathBuf },
    #[snafu(display("Writing to file failed: {}", path.display()))]
    File {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("An unknown error occurred with {}: {}", path.display(), source))]
    Unknown {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}

/// Helper method verifying permissions on a directory.
///
/// Only use on a directory. Will panic in other cases, to catch logic errors early.
pub fn is_dir_writable(path: &Path) -> Result<bool, WritableDirError> {
    assert!(path.is_dir());

    let metadata = std::fs::metadata(path).context(UnknownSnafu {
        path: path.to_path_buf(),
    })?;

    Ok(!metadata.permissions().readonly())
}

/// Ensures given path is a writable directory, following symlinks.
///
/// Will provide a useful error message if that's not the case.
pub fn ensure_writable_dir(path: &Path) -> Result<(), WritableDirError> {
    if path.is_symlink() {
        match path.canonicalize() {
            Ok(resolved_path) => {
                return ensure_writable_dir(&resolved_path)
                    .boxed()
                    .context(SymlinkSnafu {
                        path: path.to_path_buf(),
                    });
            }
            Err(e) => {
                return Err(WritableDirError::Symlink {
                    path: path.to_path_buf(),
                    source: Box::new(e),
                });
            }
        }
    } else if path.is_dir() {
        if is_dir_writable(path)? {
            return Ok(());
        } else {
            return Err(WritableDirError::Write {
                path: path.to_path_buf(),
            });
        }
    } else if path.exists() {
        return Err(WritableDirError::Type {
            path: path.to_path_buf(),
        });
    } else {
        return Err(WritableDirError::Missing {
            path: path.to_path_buf(),
        });
    }
}

/// Persist a file in a writable directory.
///
/// Will provide a useful error message if that's not the case.
/// If saving the file fails, we check that the directory is writable, otherwise an "unknown" error is returned.
pub fn save_to_writable_dir(path: &Path, content: &str) -> Result<(), WritableDirError> {
    if let Err(e) = std::fs::write(path, content) {
        ensure_writable_dir(path.parent().unwrap())?;
        return Err(WritableDirError::File {
            path: path.to_path_buf(),
            source: e,
        });
    }

    Ok(())
}

/// Copy a file in a writable directory.
///
/// Will provide a useful error message if that's not the case.
/// If saving the file fails, we check that the directory is writable, otherwise an "unknown" error is returned.
pub fn copy_to_writable_dir(source: &Path, dest: &Path) -> Result<(), WritableDirError> {
    if let Err(e) = std::fs::copy(source, dest) {
        ensure_writable_dir(dest.parent().unwrap())?;
        return Err(WritableDirError::File {
            path: dest.to_path_buf(),
            source: e,
        });
    }

    Ok(())
}
