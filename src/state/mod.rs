use hightorrent_api::{Api, ApiError, QBittorrentClient};
use rocket_dyn_templates::tera::to_value;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Config, Context, Database};

mod cache;
pub use cache::{CachedState, ReloadableState};
mod error;
pub use error::*;
mod free_space;
pub use free_space::FreeSpace;

/// FallibleState is the application state managed by Rocket.
///
/// It can be successful (when configuration, DB, and qbittorrent client are happy) or
/// unsuccessful, in which case it will contain diagnostics information.
///
/// It's a wrapper around InnerFallibleState so was can reload...
#[derive(Clone)]
pub struct FallibleState {
    pub inner: Arc<RwLock<InnerFallibleState>>,
}

impl FallibleState {
    pub async fn load() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerFallibleState::load().await)),
        }
    }

    pub async fn reload(&self) {
        let mut inner = self.inner.write().await;
        inner.reload().await;
    }

    pub async fn context(&self) -> Context {
        self.inner.read().await.context().await
    }
}

#[derive(Clone)]
pub enum InnerFallibleState {
    Ok(AppState),
    Err(AppSetupState),
}

impl InnerFallibleState {
    /// Loads the global application state.
    ///
    /// If successful, will be an Ok(AppState) variant.
    /// Otherwise, it will be an Err(AppSetupStatus) variant for user information.
    pub async fn load() -> Self {
        let config = match Config::from_cli() {
            Ok(config) => config,
            Err(e) => return Self::failed_config(vec![e]),
        };
        let database = match Database::from_dirs(&config.dirs) {
            Ok(database) => database,
            Err(e) => return Self::failed_database(e),
        };

        let host = config.qbittorrent.format_host();
        let api = match QBittorrentClient::login(
            &host,
            &config.qbittorrent.login,
            &config.qbittorrent.password,
        )
        .await
        {
            Ok(api) => api,
            Err(e) => return Self::failed_qbt(e),
        };

        Self::Ok(AppState {
            // For now hardcoded 60 seconds ttl
            free_space: CachedState::new(FreeSpace::new(&config.dirs.downloads), 60),
            config,
            database,
            api,
        })
    }

    pub async fn reload(&mut self) {
        *self = Self::load().await
    }

    pub fn failed_qbt(e: ApiError) -> Self {
        Self::Err(AppSetupState {
            inner: Arc::new(RwLock::new(InnerAppSetupState::FailedQbt(e))),
        })
    }

    pub fn failed_database(e: crate::database::DatabaseError) -> Self {
        Self::Err(AppSetupState {
            inner: Arc::new(RwLock::new(InnerAppSetupState::FailedDatabase(e))),
        })
    }

    pub fn failed_config(e: Vec<AppError>) -> Self {
        Self::Err(AppSetupState {
            inner: Arc::new(RwLock::new(InnerAppSetupState::FailedConfig(e))),
        })
    }

    pub async fn context(&self) -> Context {
        match self {
            Self::Ok(appstate) => appstate.context(),
            Self::Err(appsetupstate) => appsetupstate.context().await,
        }
    }
}

#[derive(Clone)]
pub struct AppSetupState {
    pub inner: Arc<RwLock<InnerAppSetupState>>,
}

impl AppSetupState {
    pub async fn context(&self) -> Context {
        let inner = self.inner.read().await;
        inner.context()
    }
}

/// Linear AppSuccess state
pub enum InnerAppSetupState {
    FailedConfig(Vec<AppError>),
    FailedDatabase(crate::database::DatabaseError),
    FailedQbt(ApiError),
}

impl InnerAppSetupState {
    pub fn context(&self) -> Context {
        let mut context = Context::default();

        match self {
            Self::FailedConfig(errors) => {
                for e in errors {
                    context.error_owned(e.to_html());
                }
                context.insert_bool("is_config_loaded", false);
                context.insert_bool("is_database_loaded", false);
                context.insert_bool("is_qbt_loaded", false);
            }
            Self::FailedDatabase(e) => {
                context.error_owned(e.to_string());
                context.insert_bool("is_config_loaded", true);
                context.insert_bool("is_database_loaded", false);
                context.insert_bool("is_qbt_loaded", false);
            }
            Self::FailedQbt(e) => {
                context.error_owned(e.to_string());
                context.insert_bool("is_config_loaded", true);
                context.insert_bool("is_database_loaded", true);
                context.insert_bool("is_qbt_loaded", false);
            }
        }
        context
    }
}

#[derive(Clone)]
/// The state can be cloned because it's not actually stateful, it uses inner mutability where needed
pub struct AppState {
    pub api: QBittorrentClient,
    pub config: Config,
    pub database: Database,
    pub free_space: CachedState<FreeSpace>,
}

impl AppState {
    /// Converts into a local templating Context
    pub fn context(&self) -> Context {
        let mut context = Context::default();
        context.insert_vec(
            "collections",
            self.database
                .collections()
                .iter()
                .map(|x| &x.name)
                .collect(),
        );
        context.insert("menu", to_value(2).unwrap());
        // TODO: config warning size
        let free_space = self.free_space.state();
        if free_space.bytes < 50.0 * 1024.0 * 1024.0 * 1024.0 {
            context.warning(format!("There is very little space left ({})", free_space));
        }
        context.insert_string("free_space", free_space.to_string());

        context
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }
}
