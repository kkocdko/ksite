use axum::{
    response::Html,
    routing::{get, MethodRouter},
};

async fn get_handler() -> Html<&'static [u8]> {
    Html(include_bytes!("page.html"))
}

pub fn service() -> MethodRouter {
    get(get_handler)
}
