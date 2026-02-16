use crate::state::error::*;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Path, State};
use hightorrent_api::hightorrent::{SingleTarget, Torrent, TorrentContent};
use snafu::prelude::*;

use crate::database::magnet::{self, MagnetOperator};
use crate::extractors::torrent_list::{
    TorrentListCounter, TorrentListFilter, TorrentListView, TorrentListViewRequest,
};
use crate::state::{AppState, AppStateContext, error::AppStateError};

#[derive(Template, WebTemplate)]
#[template(path = "progress.html")]
pub struct TorrentListTemplate {
    /// Global application state (errors/warnings)
    state: AppStateContext,
    /// Specific context for torrent lists.
    torrent_list: TorrentListContext,
    // Filter object
    filter: TorrentListViewRequest,
    /// Logged-in user.
    user: Option<String>,
    /// all unimported and resolved Magnets
    resolved_list_unimported: Vec<magnet::Model>,
}

#[derive(Debug)]
pub struct TorrentListContext {
    /// Number of torrents in each state (ongoing/stuck/etc)
    pub counter: TorrentListCounter,
    /// Files associated with a specific torrent.
    ///
    /// This field is Some() only when a specific torrent is selected.
    pub files: Option<Vec<TorrentContent>>,
    /// List of selected torrents.
    ///
    /// Can be a single entry when a specific torrent was requested.
    pub torrents: Vec<Torrent>,
}

pub async fn progress(
    State(app_state): State<AppState>,
    Path(view_request): Path<TorrentListViewRequest>,
) -> Result<TorrentListTemplate, AppStateError> {
    let app_state_context = app_state.context().await?;

    let resolved_list_unimported = MagnetOperator::new(app_state.clone(), None)
        .resolved_list_unimported()
        .await
        .context(MagnetUploadSnafu)?;

    // Failing to load the TorrentListView is a fatal error
    let TorrentListView {
        counter,
        filtered_list,
    } = TorrentListView::apply_request(view_request.clone(), &app_state).await?;

    // If only one torrent is inspected, display the content files
    let files = if filtered_list.len() == 1 {
        let torrent_id = &filtered_list.first().unwrap().id;
        Some(
            app_state
                .torrent_get_files(&SingleTarget::from(torrent_id))
                .await?,
        )
    } else {
        None
    };

    Ok(TorrentListTemplate {
        resolved_list_unimported,
        state: app_state_context,
        filter: view_request,
        torrent_list: TorrentListContext {
            counter,
            files,
            torrents: filtered_list,
        },
        user: None,
    })
}
