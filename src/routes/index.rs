use askama::Template;
use askama_web::WebTemplate;
use axum::response::IntoResponse;

use std::collections::HashMap;

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub collections: Vec<String>,
    pub free_space: String,
    pub errors: Vec<String>,
    pub post: HashMap<String, String>,
    pub warnings: Vec<String>,
}

pub async fn index() -> impl IntoResponse {
    IndexTemplate {
        collections: Vec::new(),
        free_space: String::from("lol"),
        errors: Vec::new(),
        post: HashMap::new(),
        warnings: Vec::new(),
    }
}
