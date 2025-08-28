use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::response::IntoResponse;

use crate::extractors::user::User;
use crate::state::AppState;

use std::collections::HashMap;

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub collections: Vec<String>,
    pub free_space: String,
    pub errors: Vec<String>,
    pub post: HashMap<String, String>,
    pub warnings: Vec<String>,
    pub user: Option<User>,
}

pub async fn index(State(app_state): State<AppState>, user: Option<User>) -> impl IntoResponse {
    IndexTemplate {
        collections: Vec::new(),
        free_space: app_state.free_space().to_string(),
        errors: Vec::new(),
        post: HashMap::new(),
        warnings: Vec::new(),
        user,
    }
}
