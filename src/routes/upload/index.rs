use rocket::State;
use rocket_dyn_templates::Template;

use crate::AppState;

#[get("/")]
/// Main upload form, when no data has been submitted yet
pub fn get(state: &State<AppState>) -> Template {
    let context = state.context();
    Template::render("upload/index", &context)
}

