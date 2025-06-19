#![deny(warnings)]

#[macro_use]
extern crate rocket;

use torrentmanager::{routes::router, state::FallibleState, templating::tera};

async fn start() -> Result<rocket::Rocket<rocket::Ignite>, rocket::Error> {
    let templating = tera();
    router()
        .manage(FallibleState::load().await)
        .attach(templating)
        .launch()
        .await
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), rocket::Error> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "torrentmanager=debug,rocket=info");
    }
    pretty_env_logger::init();

    info!("Starting TorrentManager");
    let _rocket = start().await?;

    Ok(())
}
