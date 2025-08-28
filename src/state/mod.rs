use hightorrent_api::hightorrent::TorrentList;
use hightorrent_api::{Api, QBittorrentClient};

use std::path::PathBuf;

pub mod free_space;

/// Global application state.
///
/// Used to perform queries against the system, and torrentmanager's
/// database. Can be safely cloned between threads (inner mutability).
#[derive(Clone, Debug)]
pub struct AppState {
    // TODO: multiple torrent backends
    pub torrent_client: QBittorrentClient,
}

impl AppState {
    pub async fn new() -> Self {
        // TODO: config for torrent backend
        Self {
            torrent_client: QBittorrentClient::login(
                "http://localhost:8080",
                "admin",
                "adminadmin",
            )
            .await
            .unwrap(),
        }
    }

    pub fn free_space(&self) -> free_space::FreeSpace {
        // TODO: configurable paths
        // TODO: errors
        free_space::FreeSpace::from_path(&PathBuf::from("/home"))
    }

    pub async fn torrent_list(&self) -> TorrentList {
        // TODO: errors
        self.torrent_client.list().await.unwrap()
    }
}
