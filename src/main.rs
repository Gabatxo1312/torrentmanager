#![deny(warnings)]

#[macro_use]
extern crate rocket;

use torrentmanager::{
    routes::{router, setup_router},
    state::{AppState},
    templating::tera,
};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

async fn start() -> Result<rocket::Rocket<rocket::Ignite>, rocket::Error> {
    // Custom tera templating
    let templating = tera();

    // Check if TorrentManager can be loaded successfully with the current config
    match AppState::load().await {
        Ok(state) => {
            router() // Route declarations
                .manage(state)
                .attach(templating) // Custom templating for tera
                .launch()
                .await
        }
        Err(success) => {
            setup_router()
                .manage(success)
                .attach(templating)
                .attach(torrentmanager::guards::RestartRedirect)
                .launch()
                .await
        }
    }
}

#[main]
async fn main() -> Result<(), rocket::Error> {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "torrentmanager=debug,rocket=info");
    }
    pretty_env_logger::init();
    
    info!("Starting TorrentManager");

    // Setup SIGTERM handler to stop restarting when service is stopped by systemd
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term)).unwrap();

    // Restart the web server as long as SIGTERM is not received
    while !term.load(Ordering::Relaxed) {
        info!("Starting the Rocket server");
        let _rocket = start().await?;
    }

    info!("Received SIGTERM. Stopping TorrentManager");
    Ok(())
}
