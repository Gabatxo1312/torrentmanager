use rocket::{Shutdown, State};
use rocket_dyn_templates::Template;

use crate::{AppSuccess, guards::InternalRedirect};

#[get("/<redirect>")]
/// Main upload form, when no data has been submitted yet
// pub async fn index(state: &State<AppSuccess>, restart: &State<RestartHandler>, shutdown: Shutdown, redirect: Option<InternalRedirect>) -> Template {
pub async fn index(state: &State<AppSuccess>, shutdown: Shutdown, redirect: Option<InternalRedirect>) -> Template {
    let mut context = state.context();
    if let Some(redirect) = redirect {
        context.insert_string("redirect", redirect.to_string())
    }
    
    tokio::task::spawn(async {
        // Sleep for 1s so the client can still download all assets before restart
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        shutdown.notify();
    });

    Template::render("restart", &context)
}
