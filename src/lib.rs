#![deny(warnings)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate snafu;

pub mod config;
pub mod database;
pub mod guards;
pub mod routes;
pub mod state;
pub mod templating;
pub mod utils;

// Test
pub mod series;

pub use config::Config;
pub use database::{Database, UploadID};
pub use state::{AppError, AppSetupState, AppState};
pub use templating::context::Context;
