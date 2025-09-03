use askama::Template;
use askama_web::WebTemplate;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use snafu::prelude::*;

use super::free_space::FreeSpaceError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum AppStateError {
    #[snafu(display("Failed to initialize the torrent API"))]
    InitAPI { source: hightorrent_api::ApiError },
    #[snafu(display("Failed to communicate with the torrent client"))]
    API { source: hightorrent_api::ApiError },
    #[snafu(display("Failed to get free space information"))]
    FreeSpace { source: FreeSpaceError },
}

/// Global error page generated from an [AppStateError].
#[derive(Clone, Debug, Template, WebTemplate)]
#[template(path = "error.html")]
pub struct AppStateErrorContext {
    state: AppStateErrorContextInner,
}

/// Helper struct so we can reuse base.html
/// with all it's `state.foo` expressions.
#[derive(Clone, Debug)]
pub struct AppStateErrorContextInner {
    // TODO: typed errors
    // errors: Vec<AppStateError>,
    errors: Vec<String>,
}

impl From<AppStateError> for AppStateErrorContext {
    fn from(e: AppStateError) -> Self {
        Self {
            state: AppStateErrorContextInner {
                errors: vec![e.to_string()],
            },
        }
    }
}

impl IntoResponse for AppStateError {
    fn into_response(self) -> Response {
        let error_context = AppStateErrorContext::from(self);
        (StatusCode::INTERNAL_SERVER_ERROR, error_context).into_response()
    }
}
