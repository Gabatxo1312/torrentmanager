use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;

use crate::extractors::user::User;
use crate::state::{AppState, AppStateContext, error::*};

use std::collections::HashMap;

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// Global application state (errors/warnings)
    pub state: AppStateContext,
    /// TODO: Submitted values in a POST request
    ///
    /// This happens when the request was rejected by the handler, but still
    /// wants to repopulate form data from submitted values.
    pub post: HashMap<String, String>,
    /// Logged-in user.
    pub user: Option<User>,
}

pub async fn index(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<IndexTemplate, AppStateError> {
    let app_state_context = app_state.context().await?;

    Ok(IndexTemplate {
        state: app_state_context,
        post: HashMap::new(),
        user,
    })
}
