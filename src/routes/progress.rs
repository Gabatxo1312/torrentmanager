use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Path, State};
use hightorrent_api::hightorrent::{SingleTarget, Torrent, TorrentContent};

use crate::extractors::torrent_list::{
    TorrentListCounter, TorrentListView, TorrentListViewRequest,
};
use crate::state::{AppState, AppStateContext};

#[derive(Template, WebTemplate)]
#[template(path = "progress.html")]
pub struct TorrentListTemplate {
    /// Global application state (errors/warnings)
    state: AppStateContext,
    /// Number of torrents in each state (ongoing/stuck/etc)
    counter: TorrentListCounter,
    /// Files associated with a specific torrent.
    ///
    /// This field is Some() only when a specific torrent is selected.
    files: Option<Vec<TorrentContent>>,
    /// List of selected torrents.
    ///
    /// Can be a single entry when a specific torrent was requested.
    torrents: Vec<Torrent>,
    /// Logged-in user.
    user: Option<String>,
}

pub async fn progress(
    State(app_state): State<AppState>,
    Path(view_request): Path<TorrentListViewRequest>,
) -> TorrentListTemplate {
    let TorrentListView {
        counter,
        filtered_list,
    } = TorrentListView::apply_request(view_request, &app_state).await;

    let app_state_context = app_state.context().await;

    // If only one torrent is inspected, display the content files
    let files = if filtered_list.len() == 1 {
        let torrent_id = &filtered_list.first().unwrap().id;
        Some(
            app_state
                .torrent_get_files(&SingleTarget::from(torrent_id))
                .await,
        )
    } else {
        None
    };

    TorrentListTemplate {
        state: app_state_context,
        counter,
        files,
        torrents: filtered_list,
        user: None,
    }
}
