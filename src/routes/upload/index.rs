use rocket::State;
use rocket_dyn_templates::Template;
use rocket_dyn_templates::tera::to_value;

use crate::guards::IsSetupGuard;
use crate::GlobalContext;

#[get("/?<menu>")]
/// Main upload form, when no data has been submitted yet
pub fn get(_setup: IsSetupGuard, context: &State<GlobalContext>, menu: Option<usize>) -> Template {
    let mut context = context.local();
    if let Some(menu_choice) = menu {
        context.insert("menu", to_value(menu_choice).unwrap());
    }
    Template::render("upload/index", &context)
}

