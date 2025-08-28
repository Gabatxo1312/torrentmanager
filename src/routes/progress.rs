use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Path, State};
use hightorrent_api::hightorrent::Torrent;

use crate::extractors::torrent_list::{
    TorrentListCounter, TorrentListView, TorrentListViewRequest,
};
use crate::state::AppState;

#[derive(Template, WebTemplate)]
#[template(path = "progress.html")]
pub struct TorrentListTemplate {
    counter: TorrentListCounter,
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
    let view = TorrentListView::apply_request(view_request, &app_state).await;

    TorrentListTemplate {
        counter: view.counter,
        errors: vec![],
        free_space: app_state.free_space().to_string(),
        torrents: view.filtered_list,
        user: None,
        warnings: vec![],
    }
}
