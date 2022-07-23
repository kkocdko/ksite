use crate::slot::slot;
use axum::response::{Html, IntoResponse};
use axum::routing::MethodRouter;
use axum::Router;

async fn get_handler() -> impl IntoResponse {
    // page strip. it's weak but still work fine!
    // const PAGE: &str = const_str::replace!(include_str!("page.html"), "  ", "");
    const PAGE: [&str; 2] = slot(include_str!("page.html"));
    const INFO: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));
    Html([PAGE[0], INFO, PAGE[1]].join(""))
}

pub fn service() -> Router {
    Router::new().route("/welcome", MethodRouter::new().get(get_handler))
}
