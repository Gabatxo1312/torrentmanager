use hightorrent_api::{Api, ApiError};
use hightorrent_api_qbittorrent::AsyncApiClient;
use rocket_dyn_templates::tera::to_value;

use crate::{Config, Context, Database};

mod cache;
pub use cache::{CachedState, ReloadableState};
mod error;
pub use error::*;
mod free_space;
pub use free_space::FreeSpace;
// mod restart;
// pub use restart::RestartHandler;

/// Linear AppSuccess state
#[derive(Debug)]
pub enum AppSuccess {
    FailedConfig(Vec<AppError>),
    FailedDatabase(crate::database::DatabaseError),
    FailedQbt(ApiError),
    Ok,
}

impl AppSuccess {
    /// Checks if the global state loaded successfully (config file and database)
    pub fn is_loaded(&self) -> bool {
        match self {
            AppSuccess::Ok => true,
            _ => false,
        }
    }

    pub fn context(&self) -> Context {
        let mut context = Context::new();

        match self {
            AppSuccess::Ok => {
                context.insert_bool("is_config_loaded", true);
                context.insert_bool("is_database_loaded", true);
                context.insert_bool("is_qbt_loaded", true);
            }
            AppSuccess::FailedConfig(errors) => {
                for e in errors {
                    context.error_owned(e.to_html());
                }
                context.insert_bool("is_config_loaded", false);
                context.insert_bool("is_database_loaded", false);
                context.insert_bool("is_qbt_loaded", false);
            }
            AppSuccess::FailedDatabase(e) => {
                context.error_owned(e.to_string());
                context.insert_bool("is_config_loaded", true);
                context.insert_bool("is_database_loaded", false);
                context.insert_bool("is_qbt_loaded", false);
            }
            AppSuccess::FailedQbt(e) => {
                context.error_owned(e.to_string());
                context.insert_bool("is_config_loaded", true);
                context.insert_bool("is_database_loaded", true);
                context.insert_bool("is_qbt_loaded", false);
            }
        }
        context
    }
}

pub struct AppState {
    pub api: hightorrent_api_qbittorrent::AsyncApiClient,
    pub config: Config,
    pub database: Database,
    pub free_space: CachedState<FreeSpace>,
}

impl AppState {
    /// Loads the global state from CLI config file or default ./TorrentManager.toml
    pub async fn load() -> Result<AppState, AppSuccess> {
        let config = Config::from_cli().map_err(|e| AppSuccess::FailedConfig(vec![e]))?;
        let database =
            Database::from_dirs(&config.dirs).map_err(|e| AppSuccess::FailedDatabase(e))?;

        let host = config.qbittorrent.format_host();
        let api = AsyncApiClient::login(
            &host,
            &config.qbittorrent.login,
            &config.qbittorrent.password,
        )
        .await
        .map_err(|e| AppSuccess::FailedQbt(e))?;

        Ok(AppState {
            // For now hardcoded 60 seconds ttl
            free_space: CachedState::new(FreeSpace::new(&config.dirs.downloads), 60),
            config,
            database,
            api,
        })
    }

    /// Converts into a local templating Context
    pub fn context(&self) -> Context {
        let mut context = Context::new();
        context.insert_vec(
            "collections",
            self.database
                .collections()
                .list()
                .iter()
                .map(|x| &x.name)
                .collect(),
        );
        context.insert("menu", to_value(2).unwrap());
        // TODO: config warning size
        let free_space = self.free_space.state();
        if free_space.bytes < 50.0 * 1024.0 * 1024.0 * 1024.0 {
            context.warning(format!(
                "There is very little space left ({})",
                free_space.to_string()
            ));
        }
        context.insert_string("free_space", free_space.to_string());

        context
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }
}

// pub struct QBittorrent {
//     pub api: hightorrent_api_qbittorrent::AsyncApiClient,
//     pub credentials: crate::config::QBittorrentConfig,
// }

// impl ReloadableState for QBittorrent {
//     fn reload_state(&mut self) {}
// }
