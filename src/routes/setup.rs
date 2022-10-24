use rocket::State;
use rocket_dyn_templates::Template;
use rocket_dyn_templates::tera::to_value;

use crate::GlobalContext;

#[get("/?<menu>", rank=2)]
pub fn get(context: &State<GlobalContext>, menu: Option<usize>) -> Template {
    let mut context = context.local();
    if let Some(menu_choice) = menu {
        context.insert("menu", to_value(menu_choice).unwrap());
    }
    Template::render("setup", &context)
}
