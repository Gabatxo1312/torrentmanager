use askama::Template;
use askama_web::WebTemplate;
use axum::Form;
use axum::extract::State;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use snafu::prelude::*;

// TUTORIAL: https://github.com/SeaQL/sea-orm/blob/master/examples/axum_example/
use crate::database::category::{self, CategoryOperator};
use crate::extractors::normalized_path::NormalizedPathComponent;
use crate::extractors::user::User;
use crate::state::flash_message::{OperationStatus, get_cookie};
use crate::state::{AppState, AppStateContext, error::*};

#[derive(Deserialize)]
pub struct IndexTemplateParameter {
    pub torrent_id: Option<String>,
}

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
    // QueryParameter
    pub parameter: IndexTemplateParameter,
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
    pub async fn new(
        app_state: AppState,
        user: Option<User>,
        jar: CookieJar,
        parameter: IndexTemplateParameter,
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
                parameter,
            },
        ))
    }

    pub fn category_show_url(
        category_name: &NormalizedPathComponent,
        parameter: &IndexTemplateParameter,
    ) -> String {
        if let Some(torrent_id) = &parameter.torrent_id {
            format!("/folders/{}?torrent_id={}", category_name, torrent_id)
        } else {
            format!("/folders/{}", category_name)
        }
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
    jar: CookieJar,
    Form(parameter): Form<IndexTemplateParameter>,
) -> Result<(CookieJar, IndexTemplate), AppStateError> {
    IndexTemplate::new(app_state, user, jar, parameter).await
}

pub async fn upload(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<UploadTemplate, AppStateError> {
    UploadTemplate::new(app_state, user).await
}
