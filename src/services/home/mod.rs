use axum::{
    response::Html,
    routing::{get, MethodRouter},
};

async fn get_handler() -> Html<&'static str> {
    Html(include_str!("page.html"))
}

pub fn main() -> MethodRouter {
    get(get_handler)
}
