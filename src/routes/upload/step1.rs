use rocket::State;
use rocket::form::Form;
use rocket_dyn_templates::Template;
use rocket_dyn_templates::tera::to_value;

use crate::GlobalContext;
use crate::guards::IsSetupGuard;
use super::UploadForm;

#[post("/?<menu>", data = "<form>")]
/// Second form, when a magnet link was sent
pub async fn post(_setup: IsSetupGuard, mut form: Form<UploadForm<'_>>, context: &State<GlobalContext>, menu: Option<usize>) -> Template {
    let mut context = context.local();
    if let Some(menu_choice) = menu {
        context.insert("menu", to_value(menu_choice).unwrap());
    }

    let form = form.validate(&mut context).await;

    if context.has_errors() {
        // TODO: reinject previous form in context
        Template::render("upload/index", &context)
    } else {
        let form = form.expect("Programming error: form should be valid when no errors have been detected");
        context.insert_string("collection", &form.collection);
        Template::render("upload/step1", &context)
    }
}
