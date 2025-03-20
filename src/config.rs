use rocket::serde::Deserialize;
use snafu::prelude::*;

use std::{
    env::args,
    path::{Path, PathBuf},
};

use crate::state::*;
use crate::utils::xdg_config_file;
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

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// The collections directory, relative to the configuration file
    pub collections_dir: PathBuf,
    /// The upload directory, relative to the configuration file
    pub uploads_dir: PathBuf,
    /// The qBittorrent downloads directory
    pub torrents_dir: PathBuf,
    /// The configuration for the bittorrent API client
    pub qbittorrent: QBittorrentConfig,
    /// Where to look for deleted torrents
    #[serde(default)]
    pub torrent_locations: Vec<PathBuf>,
}

impl Config {
    /// Loads config file in this order:
    ///   - if first argument: first argument or fail
    ///   - if no argument: ./TorrentManager.toml then ~/.config/TorrentManager/TorrentManager.toml
    pub fn from_cli() -> Result<Config, Vec<AppError>> {
        if let Some(p) = args().into_iter().nth(1) {
            // Only try 1st argument, or fail
            Self::from_file(p).map_err(|e| vec![e])
        } else {
            if let Some(xdgcfg) = xdg_config_file("TorrentManager.toml") {
                // Try ./TorrentManager.toml then ~/.config/TorrentManager/TorrentManager.toml
                Self::from_files(vec![PathBuf::from("./TorrentManager.toml"), xdgcfg])
            } else {
                // Just try ./TorrentManager.toml
                Self::from_file("./TorrentManager.toml").map_err(|e| vec![e])
            }
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

    /// from_files tries to lookup a list of possible config files.
    /// If a config
    pub fn from_files(paths: Vec<PathBuf>) -> Result<Config, Vec<AppError>> {
        let mut errors: Vec<AppError> = Vec::new();
        for path in &paths {
            match std::fs::read_to_string(path).context(NoConfigError {
                path: path.to_path_buf(),
            }) {
                Ok(c) => {
                    // We found a config file... if it's broken we don't want to fallback to others
                    return Self::from_str(&c, path).map_err(|e| vec![e]);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }
        Err(errors)
    }
}
