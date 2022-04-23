use axum::{
    response::Html,
    routing::{get, MethodRouter},
};

async fn get_handler() -> Html<&'static str> {
    Html(include_str!("page.html"))
}

pub fn service() -> MethodRouter {
    get(get_handler)
}
