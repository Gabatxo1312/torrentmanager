use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use snafu::prelude::*;

use crate::database::magnet;
use crate::database::magnet::MagnetOperator;
use crate::database::operation::OperationLog;
use crate::database::operation::OperationType;
use crate::extractors::user::User;
use crate::state::{AppState, AppStateContext, error::*};

#[derive(Template, WebTemplate)]
#[template(path = "logs/index.html")]
pub struct LogTemplate {
    pub state: AppStateContext,
    pub logs: Vec<OperationLog>,
    pub user: Option<User>,
    /// all unimported and resolved Magnets
    pub resolved_list_unimported: Vec<magnet::Model>,
}

pub async fn index(
    State(app_state): State<AppState>,
    user: Option<User>,
) -> Result<LogTemplate, AppStateError> {
    let app_state_context = app_state.context().await?;
    let resolved_list_unimported = MagnetOperator::new(app_state.clone(), user.clone())
        .resolved_list_unimported()
        .await
        .context(MagnetUploadSnafu)?;

    let logs = app_state.logger.read().await.context(LoggerSnafu)?;

    Ok(LogTemplate {
        state: app_state_context,
        resolved_list_unimported,
        logs,
        user,
    })
}
