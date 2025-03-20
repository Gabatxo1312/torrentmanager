use rocket::config::{Config as RocketConfig, Shutdown as RocketShutdown, Sig as RocketSig};

pub mod chores;
pub mod progress;
pub mod restart;
pub mod setup;
pub mod upload;

// Disable rocket built-in Ctrl+C so we can reload the server
pub fn rocket_config() -> RocketConfig {
    RocketConfig {
        shutdown: RocketShutdown {
            ctrlc: false,
            #[cfg(unix)]
            signals: {
                let mut set = std::collections::HashSet::new();
                // set.insert(RocketSig::Term);
                set.insert(RocketSig::Hup);
                set
            },
            grace: 10,
            mercy: 5,
            force: true,
            ..Default::default()
        },
        ..RocketConfig::default()
    }
}

/// Define the routes exposed by the server. Specific routes are implemented in submodules,
/// while the overall declarations are operated here, starting in the router function.
/// For routes which should not operate unless TorrentManager is setup properly, don't forget to add setup::index
/// to the list of routes available under that mount point, so that you're redirected to setup instead of 404.
pub fn router() -> rocket::Rocket<rocket::Build> {
    rocket::custom(&rocket_config())
        .mount("/assets", rocket::fs::FileServer::from("assets")) // Static files
        .mount("/", index()) // Homepage
        .mount("/upload", upload()) // Upload page
        .mount("/progress", progress())
        .mount("/chores", chores())
    // .mount("/setup", setup())
    // .mount("/setup/restart", routes![restart::index])
}

/// The router for when launch was not successfuly, just exposes /setup as / and /restart to
/// try and go into successful mode
pub fn setup_router() -> rocket::Rocket<rocket::Build> {
    rocket::custom(&rocket_config())
        // .attach(torrentmanager::guards::RestartRedirect)
        .mount("/assets", rocket::fs::FileServer::from("assets")) // Static files
        .mount("/setup", setup())
        .mount("/setup/restart", routes![restart::index])
}

/// The routes available for homepage
pub fn index() -> Vec<rocket::Route> {
    routes![upload::index::get,]
}

/// The routes available under /upload
pub fn upload() -> Vec<rocket::Route> {
    routes![upload::index::get, upload::step1::post, upload::step2::post,]
}

/// The routes available under /progress
pub fn progress() -> Vec<rocket::Route> {
    routes![
        progress::index,
        progress::ongoing,
        progress::stuck,
        progress::unmanaged,
        progress::hash,
    ]
}

pub fn setup() -> Vec<rocket::Route> {
    routes![setup::get,]
}

pub fn chores() -> Vec<rocket::Route> {
    routes![chores::get,]
}
