use hightorrent_api::hightorrent::{SingleTarget, TorrentContent, TorrentList};
use hightorrent_api::{Api, QBittorrentClient};

use crate::config::AppConfig;

pub mod free_space;

/// Global application state.
///
/// Used to perform queries against the system, and torrentmanager's
/// database. Can be safely cloned between threads (inner mutability).
#[derive(Clone, Debug)]
pub struct AppState {
    // Global configuration for TorrentManager
    pub config: AppConfig,

    // TODO: multiple torrent backends
    pub torrent_client: QBittorrentClient,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        // TODO: config for torrent backend
        Self {
            config,
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
        // TODO: errors
        free_space::FreeSpace::from_path(&self.config.media_dir)
    }

    pub async fn torrent_list(&self) -> TorrentList {
        // TODO: errors
        self.torrent_client.list().await.unwrap()
    }

    pub async fn torrent_get_files(&self, target: &SingleTarget) -> Vec<TorrentContent> {
        // TODO: errors
        self.torrent_client.get_files(target).await.unwrap()
    }
}
