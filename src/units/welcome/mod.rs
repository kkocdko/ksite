use axum::response::Html;
use axum::routing::MethodRouter;
use axum::Router;

async fn get_handler() -> Html<&'static str> {
    // it's weak but still work fine!
    // const PAGE: &str = const_str::replace!(include_str!("page.html"), "  ", "");
    Html(include_str!("page.html"))
}

pub fn service() -> Router {
    Router::new().route("/welcome", MethodRouter::new().get(get_handler))
}