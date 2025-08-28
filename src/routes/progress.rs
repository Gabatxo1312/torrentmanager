use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Path, State};
use hightorrent_api::hightorrent::{SingleTarget, Torrent, TorrentContent};

use crate::extractors::torrent_list::{
    TorrentListCounter, TorrentListView, TorrentListViewRequest,
};
use crate::state::AppState;

#[derive(Template, WebTemplate)]
#[template(path = "progress.html")]
pub struct TorrentListTemplate {
    counter: TorrentListCounter,
    files: Option<Vec<TorrentContent>>,
    free_space: String,
    errors: Vec<String>,
    torrents: Vec<Torrent>,
    user: Option<String>,
    warnings: Vec<String>,
}

pub async fn progress(
    State(app_state): State<AppState>,
    Path(view_request): Path<TorrentListViewRequest>,
) -> TorrentListTemplate {
    let TorrentListView {
        counter,
        filtered_list,
    } = TorrentListView::apply_request(view_request, &app_state).await;

    // If only one torrent is inspected, display the content files
    let files = if filtered_list.len() == 1 {
        let torrent_id = &filtered_list.get(0).unwrap().id;
        Some(
            app_state
                .torrent_get_files(&SingleTarget::from(torrent_id))
                .await,
        )
    } else {
        None
    };

    TorrentListTemplate {
        counter: counter,
        errors: vec![],
        files: files,
        free_space: app_state.free_space().to_string(),
        torrents: filtered_list,
        user: None,
        warnings: vec![],
    }
}
