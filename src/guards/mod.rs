use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{uri::Origin, Status};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::Data;
use rocket::request::FromParam;

use std::convert::Infallible;
use std::str::FromStr;

use crate::AppSuccess;

pub struct IsSetupGuard;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IsSetupGuard {
    type Error = Infallible;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let success = req.rocket().state::<AppSuccess>().unwrap();
        if success.is_loaded() {
            Outcome::Success(IsSetupGuard)
        } else {
            Outcome::Forward(Status::ExpectationFailed)
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
pub struct InternalRedirect(Origin<'static>);

impl<'r> FromParam<'r> for InternalRedirect {
    type Error = NoInternalRedirect;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Self::from_base64(param)
    }
}

impl InternalRedirect {
    pub fn from_base64(s: &str) -> Result<Self, NoInternalRedirect> {
        let redirect_str = String::from_utf8(
            base64_url::decode(&s).map_err(|_e| NoInternalRedirect::InvalidBase64)?
        ).map_err(|_e| NoInternalRedirect::InvalidBase64)?;

        Self::from_str(&redirect_str)        
    }
    
    pub fn as_base64(&self) -> String {
        base64_url::encode(&self.0.to_string())
    }
}

impl ToString for InternalRedirect {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for InternalRedirect {
    type Err = NoInternalRedirect;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("/") {
            let origin = Origin::parse_owned(s.to_string()).unwrap();
            Ok(InternalRedirect(origin))
        } else {
            Err(NoInternalRedirect::ExternalRedirect)
        }
    }
}

/// Redirect requests to status page, except assets.
/// 
/// This fairing is only run when normal startup is not successful in [torrentmanager::routes::start].
/// It redirects all queries to the `/setup` route, where a restart button will take the user to their
/// intended page once setup is completed.
pub struct RestartRedirect;

#[rocket::async_trait]
impl Fairing for RestartRedirect {
    fn info(&self) -> Info {
        Info {
            name: "Restart Redirect when setup is not complete",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        let success = req.rocket().state::<AppSuccess>().unwrap();
        if !success.is_loaded() {
            info!("TorrentManager not loaded yet.");
            // Encode the request URI
            let req_uri = req.uri().to_string();
            let req_path = req.uri().path();

            if ! (req_path.starts_with("/assets") || req_path.starts_with("/setup")) {
                info!("Changing request URL for /setup");
                let encoded_uri = base64_url::encode(&req_uri);
                info!("Request URI B64: {}", encoded_uri);
                let new_uri = Origin::parse_owned(format!("/setup/status/{}", encoded_uri)).unwrap();
                info!("New URL: {}", new_uri);
                req.set_uri(new_uri);
            }
        }
    }
}
