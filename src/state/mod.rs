use hightorrent_api::hightorrent::{SingleTarget, TorrentContent, TorrentList};
use hightorrent_api::{Api, QBittorrentClient};
use snafu::prelude::*;

use crate::config::AppConfig;

pub mod error;
pub mod free_space;

use error::*;

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
    pub async fn new(config: AppConfig) -> Result<Self, AppStateError> {
        // TODO: config for torrent backend

        let torrent_client =
            QBittorrentClient::new_not_logged_in("http://localhost:8080", "admin", "adminadmin")
                .context(InitAPISnafu)?;

        Ok(Self {
            config,
            torrent_client,
        })
    }

    pub fn free_space(&self) -> Result<free_space::FreeSpace, AppStateError> {
        free_space::FreeSpace::from_path(&self.config.media_dir).context(FreeSpaceSnafu)
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
