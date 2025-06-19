use rocket::State;
use rocket_dyn_templates::Template;

use crate::guards::InternalRedirect;
use crate::state::{AppSetupState, FallibleState};

#[get("/<path..>", rank = 99)]
pub async fn get(
    // state: &State<FallibleState>,
    state: AppSetupState,
    path: std::path::PathBuf,
) -> Template {
    // Prepend / so we get internal redirection
    let request_b64 = base64_url::encode(&format!("/{}", path.to_str().unwrap()));
    info!("{}", request_b64);
    let mut context = state.context().await;
    context.insert_string("redirect", request_b64);
    Template::render("setup", &context)
}

#[get("/<redirect>")]
/// Main upload form, when no data has been submitted yet
pub async fn restart(state: &State<FallibleState>, redirect: InternalRedirect) -> Template {
    state.reload().await;

    let mut context = state.context().await;
    context.insert_string("redirect", redirect.to_string());

    Template::render("restart", &context)
}
