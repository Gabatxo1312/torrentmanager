use axum::Router;
use axum::routing::get;
use axum::serve::Listener;
use static_serve::embed_assets;

pub mod extractors;
pub mod routes;
pub mod state;

pub fn router(state: state::AppState) -> Router {
    // Embed the assets in the binary, generating the static_router function
    embed_assets!("assets");

    Router::new()
        // Register dynamic routes
        .route("/", get(routes::index::index))
        .route("/progress/{view_request}", get(routes::progress::progress))
        // Register static assets routes
        .nest("/assets", static_router())
        // Allow to access global AppState from routes
        .with_state(state)
}

pub async fn serve<L>(listener: L)
where
    L: Listener,
    L::Addr: std::fmt::Debug,
{
    let state = state::AppState::new().await;
    let app = router(state);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
