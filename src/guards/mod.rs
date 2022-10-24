use rocket::request::{self, Outcome, Request, FromRequest};

use std::convert::Infallible;

use crate::GlobalContext;

pub struct IsSetupGuard;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IsSetupGuard {
    type Error = Infallible;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let global_context = req.rocket().state::<GlobalContext>().unwrap();
        if global_context.is_loaded() {
            Outcome::Success(IsSetupGuard)
        } else {
            Outcome::Forward(())
        }
    }
}
