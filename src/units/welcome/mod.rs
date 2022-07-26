use crate::include_page;
use axum::response::{Html, IntoResponse};
use axum::routing::MethodRouter;
use axum::Router;

async fn get_handler() -> impl IntoResponse {
    const PAGE: [&str; 2] = include_page!("page.html");
    const INFO: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));
    const BODY: &str = const_str::concat!(PAGE[0], INFO, PAGE[1]);
    Html(BODY)
}

pub fn service() -> Router {
    Router::new().route("/welcome", MethodRouter::new().get(get_handler))
}
