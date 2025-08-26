use axum::Router;
use axum::routing::get;
use axum::serve::Listener;
use static_serve::embed_assets;

pub mod routes;
pub mod state;

pub fn router() -> Router {
    // Embed the assets in the binary, generating the static_router function
    embed_assets!("assets");

    Router::new()
        // Register dynamic routes
        .route("/", get(routes::index::index))
        // Register static assets routes
        .nest("/assets", static_router())
        // Allow to access global AppState from routes
        .with_state(state::AppState)
}

pub async fn serve<L>(listener: L)
where
    L: Listener,
    L::Addr: std::fmt::Debug,
{
    let app = router();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
