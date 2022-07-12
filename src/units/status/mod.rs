use axum::response::Html;
use axum::routing::MethodRouter;
use axum::Router;

async fn get_handler() -> Html<&'static str> {
    // std::env::
    Html(include_str!("page.html"))
}

pub fn service() -> Router {
    Router::new().route("/status", MethodRouter::new().get(get_handler))
}
