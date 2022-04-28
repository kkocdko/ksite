use axum::Router;
use axum::{response::Html, routing::MethodRouter};

async fn get_handler() -> Html<&'static str> {
    Html(include_str!("page.html"))
}

pub fn service() -> Router {
    Router::new().route("/welcome", MethodRouter::new().get(get_handler))
}
