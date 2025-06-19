use hightorrent_api::hightorrent::SingleTarget;
use snafu::prelude::*;

use std::path::{Path, PathBuf};
use std::str::FromStr;

//use crate::database::DatabaseError;
use crate::database::*;
use crate::utils::writable_dir::*;

/// A UploadID represents a torrent/magnet hosted in the upload dir.
/// It does not need to exist, but ensures that no illegal characters
/// permit path traversal ('/').
#[derive(Clone, Debug)]
//pub struct UploadID(String);
pub struct UploadID {
    hash: String,
    kind: UploadType,
    complete: UnicodePath,
}

#[derive(Clone, Debug)]
pub enum UploadType {
    Magnet,
    Torrent,
}

impl AsRef<str> for UploadType {
    fn as_ref(&self) -> &'static str {
        match self {
            UploadType::Magnet => ".magnet",
            UploadType::Torrent => ".torrent",
        }
    }
}

impl AsRef<str> for UploadID {
    fn as_ref(&self) -> &str {
        self.complete.as_str()
    }
}

impl AsRef<Path> for UploadID {
    fn as_ref(&self) -> &Path {
        self.complete.as_path()
    }
}

impl std::str::FromStr for UploadID {
    type Err = DatabaseError;

    /// Used for passing UploadID around the system in stringy types (eg. in HTTP forms)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with(".torrent") {
            let hash = s.trim_end_matches(".torrent");
            Self::torrent(hash)
        } else if s.ends_with(".magnet") {
            let hash = s.trim_end_matches(".magnet");
            Self::magnet(hash)
        } else {
            Err(DatabaseError::InvalidContentID { id: s.to_string() })
        }
    }
}

impl UploadID {
    fn new(hash: &str, kind: UploadType) -> Result<UploadID, DatabaseError> {
        // TODO: make infohash typesafe by checking valid sha1/sha256...
        // maybe in torrent crate? Also make it lowercase?!
        if hash.contains('/') {
            Err(DatabaseError::InvalidContentID {
                id: hash.to_string(),
            })
        } else {
            let mut complete_str = hash.to_lowercase();
            complete_str.push_str(kind.as_ref());
            Ok(UploadID {
                hash: hash.to_string(),
                kind,
                complete: UnicodePath::new(&complete_str),
            })
        }
    }

    pub fn magnet(hash: &str) -> Result<UploadID, DatabaseError> {
        Self::new(hash, UploadType::Magnet)
    }

    pub fn torrent(hash: &str) -> Result<UploadID, DatabaseError> {
        Self::new(hash, UploadType::Torrent)
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.complete.to_path_buf()
    }

    pub fn to_single_target(&self) -> SingleTarget {
        SingleTarget::from_str(self.hash()).unwrap()
    }

    //pub fn truncated_hash(&self) -> TruncatedHash {
    // safe unwrap... TODO: maybe make it more explicit in function signature?
    //TruncatedHash::from_str(self.hash()).unwrap()
    //}

    pub fn single_target(&self) -> SingleTarget {
        // safe unwrap
        SingleTarget::from_str(self.hash()).unwrap()
    }

    pub async fn add_to(
        &self,
        state: &AppState,
        final_path: UploadPath,
        collection: &str,
    ) -> Result<(), hightorrent_api::ApiError> {
        // TODO: make better
        let tags = vec![
            base64_url::unescape(&base64_url::encode(final_path.as_ref())).to_string(),
            collection.to_string(),
        ];
        let path = state.database.uploads().upload_path(self);
        let save_path = state.database.save_paths().compute(self);

        let add = match self.kind {
            UploadType::Magnet => state
                .api
                .add()
                .magnet_file(&path)
                .tags(tags)
                .save_path(save_path.as_ref()),
            UploadType::Torrent => state
                .api
                .add()
                .torrent_file(&path)
                .tags(tags)
                .save_path(save_path.as_ref()),
        };

        add.send().await
    }

    pub fn to_lowercase(&self) -> String {
        self.complete.to_string().to_lowercase()
    }
}

impl std::fmt::Display for UploadID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.complete)
    }
}

fn doesnt_traverse_path(path: &str) -> bool {
    !(path.starts_with('/') || path.contains("../"))
}

fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        String::from(".")
    } else {
        path.to_string()
    }
}

fn complete_path(basedir: &str, targetdir: &str) -> String {
    match (basedir, targetdir) {
        (".", ".") => String::from("."),
        (".", _) => format!("./{targetdir}"),
        (_, ".") => basedir.to_string(),
        (_, _) => format!("{basedir}/{targetdir}"),
    }
}

#[derive(Debug)]
/// Where an uploaded torrent/magnet will be archived, relative to the [`Database`]'s upload_dir
pub struct UploadPath(String);
//pub struct UploadPath<'a> {
//    basedir: &'a Collection,
//    targetdir: &'a str,
//}
//
//impl<'a> UploadPath<'a> {
//    pub fn from_dirs(basedir: &'a Collection, targetdir: &'a str) -> UploadPath<'a> {
//
//    }
//
//    pub fn to_string(&'a self) -> String {
//        self.complete_path()
//    }
//
//    pub fn complete_path(&'a self) -> String {
//
//    }
//}
//
impl UploadPath {
    pub fn from_str(basedir: &str, targetdir: &str) -> Option<UploadPath> {
        if !doesnt_traverse_path(basedir) || !doesnt_traverse_path(targetdir) {
            None
        } else {
            // TODO verify db collection has folder basedir?
            let basedir = normalize_path(basedir);
            let targetdir = normalize_path(targetdir);
            let complete_path = complete_path(&basedir, &targetdir);
            Some(UploadPath(complete_path))
        }
    }

    pub fn absolute(&self, upload_dir: &Path) -> PathBuf {
        upload_dir.join(&self.0)
    }
}

impl std::fmt::Display for UploadPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl AsRef<str> for UploadPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct UploadDB {
    basedir: PathBuf,
}

impl UploadDB {
    pub fn from(basedir: &Path) -> UploadDB {
        UploadDB {
            basedir: basedir.to_path_buf(),
        }
    }

    pub fn upload_path(&self, entry: &UploadID) -> PathBuf {
        self.basedir.join(entry)
    }

    pub fn has_upload(&self, entry: &UploadID) -> Result<bool, DatabaseError> {
        let path = self.upload_path(entry);
        Ok(path.is_file())
    }

    pub fn persist_upload(&self, entry: &UploadID, content: &str) -> Result<(), DatabaseError> {
        if self.has_upload(entry)? {
            println!("Entry already present: {entry}");
            Ok(())
        } else {
            let path = self.upload_path(entry);
            save_to_writable_dir(&path, content).context(PersistUploadSnafu)?;
            Ok(())
        }
    }

    pub fn persist_upload_copy(
        &self,
        entry: &UploadID,
        source: &Path,
    ) -> Result<(), DatabaseError> {
        if self.has_upload(entry)? {
            println!("Entry already present: {entry}");
            Ok(())
        } else {
            let path = self.upload_path(entry);
            copy_to_writable_dir(source, &path).context(PersistUploadSnafu)?;
            Ok(())
        }
    }
}
