use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::State;
use rocket_dyn_templates::Template;

use crate::{guards::InternalRedirect, AppSuccess};

pub struct Grace(pub u32);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Grace {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let grace = req.rocket().config().shutdown.grace;
        Outcome::Success(Grace(grace))
    }
}

#[get("/status/<redirect>", rank = 2)]
pub fn get(
    grace: Grace,
    state: &State<AppSuccess>,
    redirect: Option<InternalRedirect>,
) -> Template {
    warn!("ASKING SHUTDOWN: {}", grace.0);
    let mut context = state.context();

    if let Some(redirect) = redirect {
        context.insert_string("redirect", redirect.as_base64())
    }
    Template::render("setup", &context)
}
