#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_dyn_templates;

pub mod config;
pub mod database;
pub mod guards;
pub mod routes;
pub mod templating;
pub mod utils;

pub use config::Config;
pub use database::Database;
pub use templating::context::{Context, GlobalContext};

#[launch]
fn rocket() -> _ {
    let config = Config::default();
    let context = GlobalContext::from_config(&config);

    routes::router() // Route declarations
        .manage(context) // Global context as shared State
        .attach(templating::tera()) // Custom templating for tera
}
