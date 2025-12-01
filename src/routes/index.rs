use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use snafu::prelude::*;

// TUTORIAL: https://github.com/SeaQL/sea-orm/blob/master/examples/axum_example/
use crate::database::category::{self, CategoryOperator};
use crate::extractors::user::User;
use crate::state::{AppState, AppStateContext, error::*};

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// Global application state (errors/warnings)
    pub state: AppStateContext,
    /// Logged-in user.
    pub user: Option<User>,
    /// Categories
    pub categories: Vec<category::Model>,
}

#[derive(Template, WebTemplate)]
#[template(path = "upload.html")]
pub struct UploadTemplate {
    /// Global application state (errors/warnings)
    pub state: AppStateContext,
    /// Logged-in user.
    pub user: Option<User>,
    /// Categories
    pub categories: Vec<String>,
}

impl IndexTemplate {
    pub async fn new(app_state: AppState, user: Option<User>) -> Result<Self, AppStateError> {
        let app_state_context = app_state.context().await?;

        let categories = CategoryOperator::new(app_state.clone(), user.clone())
            .list()
            .await
            .context(CategorySnafu)?;

        Ok(IndexTemplate {
            state: app_state_context,
            user,
            categories,
        })
    }
}

impl UploadTemplate {
    pub async fn new(app_state: AppState, user: Option<User>) -> Result<Self, AppStateError> {
        let categories: Vec<String> = CategoryOperator::new(app_state.clone(), user.clone())
            .list()
            .await
            .context(CategorySnafu)?
            .into_iter()
            .map(|x| x.name.to_string())
            .collect();

        Ok(UploadTemplate {
            state: app_state.context().await?,
            user,
            categories,
        })
    }
}

pub async fn index(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<IndexTemplate, AppStateError> {
    IndexTemplate::new(app_state, user).await
}

pub async fn upload(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<UploadTemplate, AppStateError> {
    UploadTemplate::new(app_state, user).await
}
