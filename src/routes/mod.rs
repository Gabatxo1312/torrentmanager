pub mod setup;
pub mod upload;

/// Define the routes exposed by the server. Specific routes are implemented in submodules,
/// while the overall declarations are operated here, starting in the router function.
/// For routes which should not operate unless TorrentManager is setup properly, don't forget to add setup::index
/// to the list of routes available under that mount point, so that you're redirected to setup instead of 404.
pub fn router() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/assets", rocket::fs::FileServer::from("assets")) // Static files
        .mount("/", index()) // Homepage
        .mount("/upload", upload()) // Upload page
}

/// The routes available for homepage
pub fn index() -> Vec<rocket::Route> {
    routes![
        setup::get,
        upload::index::get,
    ]
}

/// The routes available under /upload
pub fn upload() -> Vec<rocket::Route> {
    routes![
        setup::get,
        upload::index::get,
        upload::step1::post,
    ]
}
