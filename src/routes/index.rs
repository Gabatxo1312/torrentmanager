use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum_extra::extract::CookieJar;
use snafu::prelude::*;

// TUTORIAL: https://github.com/SeaQL/sea-orm/blob/master/examples/axum_example/
use crate::database::category::{self, CategoryOperator};
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
}

impl IndexTemplate {
    pub async fn new(
        app_state: AppState,
        user: Option<User>,
        jar: CookieJar,
    ) -> Result<(CookieJar, Self), AppStateError> {
        let app_state_context = app_state.context().await?;

        let categories = CategoryOperator::new(app_state.clone(), user.clone())
            .list()
            .await
            .context(CategorySnafu)?;

        let (jar, operation_status) = get_cookie(jar);

        Ok((
            jar,
            IndexTemplate {
                state: app_state_context,
                user,
                categories,
                flash: operation_status,
            },
        ))
    }
}

pub async fn index(
    State(app_state): State<AppState>,
    user: Option<User>,
    jar: CookieJar,
) -> Result<(CookieJar, IndexTemplate), AppStateError> {
    IndexTemplate::new(app_state, user, jar).await
}
