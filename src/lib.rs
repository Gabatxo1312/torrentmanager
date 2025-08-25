use axum::Router;
use axum::routing::get;
use axum::serve::Listener;

pub fn router() -> Router {
    Router::new()
        .route("/", get(|| async { format!("Hello from axum") }))
}

pub async fn serve<L>(listener: L)
where 
    L: Listener,
    L::Addr: std::fmt::Debug
{
    let app = router().into_make_service();
    axum::serve(listener, app).await.unwrap();
}
