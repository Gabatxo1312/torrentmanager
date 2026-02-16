use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum_extra::extract::CookieJar;
use snafu::prelude::*;

// TUTORIAL: https://github.com/SeaQL/sea-orm/blob/master/examples/axum_example/
use crate::database::category::{self, CategoryOperator};
use crate::database::magnet::{self, MagnetOperator};
use crate::extractors::user::User;
use crate::routes::magnet::MagnetForm;
use crate::state::flash_message::{OperationStatus, get_cookie};
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
    /// Operation status for UI confirmation
    pub flash: Option<OperationStatus>,
    /// all unimported and resolved Magnets
    pub resolved_list_unimported: Vec<magnet::Model>,
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
    // TODO: also support torrent upload
    /// Magnet upload form
    pub post: Option<MagnetForm>,
    /// Error with submitted magnet
    pub post_error: Option<AppStateError>,
    /// all unimported and resolved Magnets
    pub resolved_list_unimported: Vec<magnet::Model>,
}

impl IndexTemplate {
    pub async fn new(
        app_state: AppState,
        user: Option<User>,
        jar: CookieJar,
    ) -> Result<(CookieJar, Self), AppStateError> {
        let app_state_context = app_state.context().await?;
        let resolved_list_unimported = MagnetOperator::new(app_state.clone(), user.clone())
            .resolved_list_unimported()
            .await
            .context(MagnetUploadSnafu)?;

        let categories = CategoryOperator::new(app_state.clone(), user.clone())
            .list()
            .await
            .context(CategorySnafu)?;

        let (jar, operation_status) = get_cookie(jar);

        Ok((
            jar,
            IndexTemplate {
                state: app_state_context,
                resolved_list_unimported,
                user,
                categories,
                flash: operation_status,
            },
        ))
    }
}

impl UploadTemplate {
    pub async fn new(app_state: AppState, user: Option<User>) -> Result<Self, AppStateError> {
        let resolved_list_unimported = MagnetOperator::new(app_state.clone(), user.clone())
            .resolved_list_unimported()
            .await
            .context(MagnetUploadSnafu)?;

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
            resolved_list_unimported,
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

pub async fn index(
    State(app_state): State<AppState>,
    user: Option<User>,
    jar: CookieJar,
) -> Result<(CookieJar, IndexTemplate), AppStateError> {
    IndexTemplate::new(app_state, user, jar).await
}

pub async fn upload(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<UploadTemplate, AppStateError> {
    UploadTemplate::new(app_state, user).await
}
