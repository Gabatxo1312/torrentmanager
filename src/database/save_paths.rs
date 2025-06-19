use std::path::{Path, PathBuf};

use crate::database::*;

/// A destination save path where the torrent client will save the actual content.
/// It is built from an [`UploadID`] (which contains the torrent hash), and a [`Database`]'s
/// `torrents_dir` attribute, generating a sort of content-addressed storage.
/// Create an actual [`TorrentSavePath`] using [`Database::torrent_dest_path`].
pub struct TorrentSavePath(String);

impl TorrentSavePath {
    /// Access the underlying string slice as a reference
    pub fn as_ref(&self) -> &str {
        &self.0
    }
}

pub struct SavePathDB<'a> {
    basedir: &'a Path,
}

impl<'a> SavePathDB<'a> {
    pub fn from(basedir: &'a Path) -> SavePathDB<'a> {
        SavePathDB { basedir }
    }

    /// Calculates the path where qBittorrent will store a torrent... by taking the hash
    pub fn compute(&self, id: &UploadID) -> TorrentSavePath {
        let path = PathBuf::from(id.to_lowercase());
        let hash = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split('.')
            .next()
            .unwrap();
        TorrentSavePath(self.basedir.join(hash).to_str().unwrap().to_string())
    }
}
