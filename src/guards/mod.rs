use rocket::http::Status;
use rocket::request::FromParam;
use rocket::request::{self, FromRequest, Outcome, Request};

use std::convert::Infallible;
use std::str::FromStr;

use crate::state::{AppSetupState, AppState, FallibleState, InnerFallibleState};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AppState {
    type Error = Infallible;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let state = req.rocket().state::<FallibleState>().unwrap();
        match &*state.inner.read().await {
            InnerFallibleState::Ok(state) => Outcome::Success(state.clone()),
            _ => Outcome::Forward(Status::ExpectationFailed),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AppSetupState {
    type Error = Infallible;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let state = req.rocket().state::<FallibleState>().unwrap();
        match &*state.inner.read().await {
            InnerFallibleState::Err(status) => Outcome::Success(status.clone()),
            _ => Outcome::Forward(Status::ExpectationFailed),
        }
    }
}

/// No [InternalRedirect] was found in request.
#[derive(Debug)]
pub enum NoInternalRedirect {
    /// The redirect is to an external resource (does not start with `/`)
    ExternalRedirect,
    /// The query parameter `redirect` was empty
    NoRedirect,
    /// Invalid base64 `redirect` value
    InvalidBase64,
}

/// A redirect to another internal page in the query param `redirect`.
///
/// Cannot represent an external redirect ; it will produce [NoInternalRedirect::ExternalRedirect] instead.
pub struct InternalRedirect(String);

impl<'r> FromParam<'r> for InternalRedirect {
    type Error = NoInternalRedirect;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Self::from_base64(param)
    }
}

impl InternalRedirect {
    pub fn from_base64(s: &str) -> Result<Self, NoInternalRedirect> {
        let redirect_str = String::from_utf8(
            base64_url::decode(&s).map_err(|_e| NoInternalRedirect::InvalidBase64)?,
        )
        .map_err(|_e| NoInternalRedirect::InvalidBase64)?;

        Self::from_str(&redirect_str)
    }

    pub fn as_base64(&self) -> String {
        base64_url::encode(&self.0.to_string())
    }
}

impl std::fmt::Display for InternalRedirect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl FromStr for InternalRedirect {
    type Err = NoInternalRedirect;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("/") {
            Ok(InternalRedirect(s.to_string()))
        } else {
            Err(NoInternalRedirect::ExternalRedirect)
        }
    }
}
