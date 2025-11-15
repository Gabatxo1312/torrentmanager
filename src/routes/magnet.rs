use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Form, Path, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use crate::database::category::{self, CategoryOperator};
use crate::database::magnet::{MagnetOperator, Model as Magnet};
use crate::extractors::user::User;
use crate::state::{AppState, AppStateContext, error::*};

/// Multipart form submitted to /magnet/upload:
///
/// - magnet: the magnet link to upload
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MagnetForm {
    pub magnet: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "magnet/show.html")]
pub struct MagnetTemplate {
    /// Global application state (errors/warnings)
    pub state: AppStateContext,
    /// Logged-in user.
    pub user: Option<User>,
    /// Parsed magnet from form
    pub magnet: Magnet,
}

pub async fn show(
    State(app_state): State<AppState>,
    user: Option<User>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppStateError> {
    let operator = MagnetOperator::new(app_state.clone(), user.clone());
    let magnet = operator.get(id).await.boxed().context(OtherSnafu)?;

    Ok(MagnetTemplate {
        state: app_state.context().await?,
        user,
        magnet,
    })
}

pub async fn upload(
    State(app_state): State<AppState>,
    user: Option<User>,
    Form(form): Form<MagnetForm>,
) -> Result<Response, AppStateError> {
    // Parse magnet
    let operator = MagnetOperator::new(app_state.clone(), user.clone());

    match operator.create(&form).await.context(MagnetUploadSnafu) {
        Ok(magnet_model) => Ok(MagnetTemplate {
            state: app_state.context().await?,
            user,
            magnet: magnet_model,
        }
        .into_response()),
        Err(e) => Ok(UploadMagnetTemplate::new(app_state, user)
            .await?
            .with_errored_form(form, e)
            .into_response()),
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "magnet/list.html")]
pub struct MagnetListTemplate {
    /// Global application state (errors/warnings)
    pub state: AppStateContext,
    /// Logged-in user.
    pub user: Option<User>,
    /// Magnets stored in database
    pub magnets: Vec<Magnet>,
}

pub async fn list(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<impl IntoResponse, AppStateError> {
    let operator = MagnetOperator::new(app_state.clone(), user.clone());
    let magnets = operator.list().await.boxed().context(OtherSnafu)?;

    Ok(MagnetListTemplate {
        state: app_state.context().await?,
        user,
        magnets,
    })
}

// TODO: remove when new upload form is ready
// or remove index.rs::UploadTemplate and templates/upload.html
#[derive(Template, WebTemplate)]
#[template(path = "magnet/upload.html")]
pub struct UploadMagnetTemplate {
    /// Global application state (errors/warnings)
    pub state: AppStateContext,
    /// Logged-in user.
    pub user: Option<User>,
    /// Magnet upload form
    pub post: Option<MagnetForm>,
    /// Error with submitted magnet
    pub post_error: Option<AppStateError>,
    pub categories: Vec<category::Model>,
}

// TODO: remove when new upload form is ready
// or remove index.rs::UploadTemplate and templates/upload.html
pub async fn get_upload(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<UploadMagnetTemplate, AppStateError> {
    UploadMagnetTemplate::new(app_state, user).await
}

// TODO: remove when new upload form is ready
// or remove index.rs::UploadTemplate and templates/upload.html
impl UploadMagnetTemplate {
    pub async fn new(app_state: AppState, user: Option<User>) -> Result<Self, AppStateError> {
        let app_state_context = app_state.context().await?;

        let categories = CategoryOperator::new(app_state.clone(), user.clone())
            .list()
            .await
            .context(CategorySnafu)?;

        Ok(UploadMagnetTemplate {
            state: app_state_context,
            user,
            categories,
            post: None,
            post_error: None,
        })
    }

    pub fn with_errored_form(mut self, form: MagnetForm, error: AppStateError) -> Self {
        self.post = Some(form);
        self.post_error = Some(error);
        self
    }
}
