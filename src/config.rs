use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use snafu::prelude::*;
use tokio::fs::{create_dir_all, read, try_exists, write};
use xdg::BaseDirectories;

use std::sync::{Arc, RwLock};

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display(
        "Failed to find configuration file: {path}\nhelp: Run `torrentmanager config generate` to create a default configuration file."
    ))]
    NoXDGConfigFile { path: Utf8PathBuf },
    #[snafu(display("Failed to read configuration file: {path} (see errors below)"))]
    FailedReadConfig {
        path: Utf8PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Failed to interpret configuration file: {path} (see errors below)"))]
    FailedParseConfig {
        path: Utf8PathBuf,
        source: toml::de::Error,
    },
    #[snafu(display("Failed to create configuration directory: {path} (see errors below)"))]
    FailedConfigDir {
        path: Utf8PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("An unknown IO error occurred (see errors below)"))]
    FailedIO { source: std::io::Error },
}

/// TorrentManager configuration file.
///
/// By default, loaded from $XDG_CONFIG_DIR/torrentmanager/config.toml,
/// that is usually ~/.config/torrentmanager/config.toml
///
/// The media directory:
///
/// - is assumed to be in a single disk/partition for free
///   space calculation
/// - has a .sources hidden directory used by the torrent client
///   to store files for seeding
/// - contains the directories for the different categories
///
/// What is currently not configurable:
///
/// - where magnets/torrents uploaded to TorrentManager are stored, hardcoded
///   to $XDG_DATA_DIR/torrentmanager/uploads
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    /// XDG configuration to produce standard paths for config/db.
    ///
    /// This is not saved in the file.
    #[serde(skip, default = "AppConfig::xdg_base_directories")]
    pub xdg_base_directories: BaseDirectories,

    /// Path to the configuration file itself.
    ///
    /// This is not saved in the file, but added manually after loading.
    #[serde(skip, default)]
    config_path: Utf8PathBuf,

    /// Main directory where content files are stored
    pub media_dir: Utf8PathBuf,

    /// Main categories to store content
    // pub media_categories: Arc<RwLock<Vec<CategoryConfig>>>
    pub media_categories: Arc<RwLock<Vec<CategoryConfig>>>,
}

impl AppConfig {
    pub fn xdg_base_directories() -> BaseDirectories {
        BaseDirectories::with_prefix("torrentmanager")
    }

    pub async fn load_from_xdg() -> Result<Self, ConfigError> {
        // Will not panic unless $HOME isn't set
        let config_dir = Self::xdg_base_directories().get_config_home().unwrap();

        // Ensure we have valid UTF8 in the path
        let config_dir = Utf8PathBuf::from_path_buf(config_dir).unwrap();

        create_dir_all(&config_dir)
            .await
            .context(FailedConfigDirSnafu {
                path: config_dir.to_path_buf(),
            })?;

        let config_path = config_dir.join("config.toml");

        if !try_exists(&config_path).await.context(FailedIOSnafu)? {
            return Err(ConfigError::NoXDGConfigFile { path: config_path });
        }

        Self::load(&config_path).await
    }

    pub async fn load(path: &Utf8Path) -> Result<Self, ConfigError> {
        // TODO: errors
        let content = read(path).await.context(FailedReadConfigSnafu {
            path: path.to_path_buf(),
        })?;
        toml::from_slice(&content).context(FailedParseConfigSnafu {
            path: path.to_path_buf(),
        })
    }

    pub async fn save(&self) {
        // TODO: errors
        let content = toml::to_string_pretty(&self).unwrap();
        write(&self.config_path, &content).await.unwrap();
    }
}

/// Configuration for a content category, such as "Video" or "ISO".
///
/// Each category has a unique identifier (usize), a human-readable name,
/// and an on-disk path.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CategoryConfig {
    /// Unique identifier. May not be changed.
    id: usize,
    /// Human-readable name for the category.
    name: String,
    /// Path where to store the content symlinks.
    ///
    /// This path is relative from the global `media_dir`, to allow moving
    /// the entire collection.
    path: Utf8PathBuf,
}

impl CategoryConfig {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Utf8Path {
        &self.path
    }
}
