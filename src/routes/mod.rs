pub mod chores;
pub mod progress;
pub mod setup;
pub mod upload;

/// Define the routes exposed by the server. Specific routes are implemented in submodules,
/// while the overall declarations are operated here, starting in the router function.
/// For routes which should not operate unless TorrentManager is setup properly, don't forget to add setup::index
/// to the list of routes available under that mount point, so that you're redirected to setup instead of 404.
pub fn router() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/assets", rocket::fs::FileServer::from("assets")) // Static files
        .mount("/", routes![upload::index::get, setup::get]) // Homepage
        .mount(
            "/upload",
            routes![upload::index::get, upload::step1::post, upload::step2::post,],
        ) // Upload page
        .mount(
            "/progress",
            routes![
                progress::index,
                progress::ongoing,
                progress::stuck,
                progress::unmanaged,
                progress::hash,
            ],
        )
        .mount("/chores", routes![chores::get,])
        .mount("/restart", routes![setup::restart])
}
