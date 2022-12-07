//! Online clipboard.

use crate::auth::auth_layer;
use crate::utils::html_escape;
use crate::{db, include_page};
use axum::extract::Path;
use axum::middleware;
use axum::response::{Html, Redirect};
use axum::routing::MethodRouter;
use axum::Router;

fn db_init() {
    db! {"
        CREATE TABLE IF NOT EXISTS paste
        (id INTEGER PRIMARY KEY, data TEXT)
    "}
    .unwrap();
}
fn db_set(id: u64, data: String) {
    db! {"
        REPLACE INTO paste
        VALUES (?1, ?2)
    ", [id, data]}
    .unwrap();
}
fn db_get(id: u64) -> Option<(String,)> {
    db! {"
        SELECT data FROM paste
        WHERE id = ?
    ", [id], ^(0)}
    .ok()
}

async fn get_handler(Path(id): Path<u64>) -> Html<String> {
    const PAGE: [&str; 2] = include_page!("page.html");
    let mut body = String::new();
    body.push_str(PAGE[0]);
    body.push_str(match &db_get(id) {
        Some(v) => &v.0,
        None => "New entry",
    });
    body.push_str(PAGE[1]);
    Html(body)
}

async fn post_handler(Path(id): Path<u64>, body: String) {
    db_set(id, html_escape(&body));
}

pub fn service() -> Router {
    db_init();
    Router::new()
        .route(
            "/paste",
            MethodRouter::new().get(|| async { Redirect::to("/paste/1") }),
        )
        .route(
            "/paste/:id",
            MethodRouter::new()
                .get(get_handler)
                .post(post_handler)
                .layer(middleware::from_fn(auth_layer)),
        )
}
