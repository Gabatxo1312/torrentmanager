use rocket::serde::Deserialize;
use snafu::prelude::*;

use std::{
    env::args,
    path::{Path, PathBuf},
};

use crate::state::*;
use crate::utils::xdg::{config_file, data_dir};
use crate::AppError;

#[derive(Clone, Debug, Deserialize)]
pub struct QBittorrentConfig {
    pub host: String,
    pub port: usize,
    pub login: String,
    pub password: String,
    pub default_location: Option<String>,
}

impl QBittorrentConfig {
    pub fn format_host(&self) -> String {
        if self.host.starts_with("http") {
            format!("{}:{}", self.host, self.port)
        } else {
            format!("http://{}:{}", self.host, self.port)
        }
    }
}

impl std::default::Default for QBittorrentConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            login: "admin".to_string(),
            password: "adminadmin".to_string(),
            default_location: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct DirConfig {
    /// Collections directory containing symlinks to the collections.
    ///
    /// For example, ~/.local/share/torrentmanager/collections/séries -> /media/Vidéo/Séries/
    ///
    /// Determines the valid collections for new uploaded torrents, as well as the final directory
    /// where to "extract" the torrent contents.
    pub collections: PathBuf,
    /// Upload directory where to store uploaded torrents/magnets.
    pub uploads: PathBuf,
    /// qBittorrent downloads directory, where torrents are stored in a folder named after their hash.
    pub downloads: PathBuf,
}

impl std::default::Default for DirConfig {
    fn default() -> DirConfig {
        let data_dir = data_dir();
        DirConfig {
            collections: data_dir.join("collections"),
            uploads: data_dir.join("uploads"),
            downloads: data_dir.join("downloads"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// Configuration for persistent directories
    #[serde(default)]
    pub dirs: DirConfig,
    /// Configuration for the QBittorrent API client
    pub qbittorrent: QBittorrentConfig,
}

impl Config {
    /// Loads config file in this order:
    ///   - if first argument: first argument or fail
    ///   - if no argument: ~/.local/share/torrentmanager/TorrentManager.toml or ./TorrentManager.toml
    pub fn from_cli() -> Result<Config, AppError> {
        if let Some(p) = args().into_iter().nth(1) {
            // Only try 1st argument, or fail
            Self::from_file(p)
        } else {
            Self::from_file(config_file("TorrentManager.toml"))
        }
    }

    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<Config, AppError> {
        let path = path.as_ref();
        toml::from_str(&std::fs::read_to_string(path).context(NoConfigError {
            path: path.to_path_buf(),
        })?)
        .context(BrokenConfigError {
            path: path.to_path_buf(),
        })
    }

    pub fn from_str(content: &str, identifier: &Path) -> Result<Config, AppError> {
        toml::from_str(content).context(BrokenConfigError {
            path: identifier.to_path_buf(),
        })
    }
}
