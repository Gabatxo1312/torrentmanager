use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Form, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use crate::database::category::{self, CategoryOperator};
use crate::database::magnet::{self, MagnetOperator, Model as Magnet};
use crate::extractors::user::User;
use crate::state::{AppState, AppStateContext, error::*};

/// Multipart form submitted to /magnet/upload:
///
/// - magnet: the magnet link to upload
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MagnetForm {
    pub magnet: String,
}

pub async fn upload(
    State(app_state): State<AppState>,
    user: Option<User>,
    Form(form): Form<MagnetForm>,
) -> Result<Response, AppStateError> {
    let resolved_list_unimported = MagnetOperator::new(app_state.clone(), user.clone())
        .resolved_list_unimported()
        .await
        .context(MagnetUploadSnafu)?;

    // Parse magnet
    let operator = MagnetOperator::new(app_state.clone(), user.clone());

    match operator.create(&form).await.context(MagnetUploadSnafu) {
        Ok(_magnet_model) => {
            let operator = MagnetOperator::new(app_state.clone(), user.clone());
            let magnets = operator.list().await.boxed().context(OtherSnafu)?;
            Ok(MagnetListTemplate {
                resolved_list_unimported,
                state: app_state.context().await?,
                user,
                magnets,
            }
            .into_response())
        }
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
    /// all unimported and resolved Magnets
    pub resolved_list_unimported: Vec<magnet::Model>,
}

pub async fn list(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<impl IntoResponse, AppStateError> {
    let resolved_list_unimported = MagnetOperator::new(app_state.clone(), user.clone())
        .resolved_list_unimported()
        .await
        .context(MagnetUploadSnafu)?;

    let operator = MagnetOperator::new(app_state.clone(), user.clone());
    let magnets = operator.list().await.boxed().context(OtherSnafu)?;

    Ok(MagnetListTemplate {
        resolved_list_unimported,
        state: app_state.context().await?,
        user,
        magnets,
    })
}

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
    /// all unimported and resolved Magnets
    pub resolved_list_unimported: Vec<magnet::Model>,
}

pub async fn get_upload(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<UploadMagnetTemplate, AppStateError> {
    UploadMagnetTemplate::new(app_state, user).await
}

impl UploadMagnetTemplate {
    pub async fn new(app_state: AppState, user: Option<User>) -> Result<Self, AppStateError> {
        let app_state_context = app_state.context().await?;

        let resolved_list_unimported = MagnetOperator::new(app_state.clone(), user.clone())
            .resolved_list_unimported()
            .await
            .context(MagnetUploadSnafu)?;

        let categories = CategoryOperator::new(app_state.clone(), user.clone())
            .list()
            .await
            .context(CategorySnafu)?;

        Ok(UploadMagnetTemplate {
            resolved_list_unimported,
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
