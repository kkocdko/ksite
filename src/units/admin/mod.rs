//! Admin console.

use crate::{db, include_page};
use axum::body::Bytes;
use axum::extract::RawQuery;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};

fn db_init() {
    // db!("VACUUM");
    db! {"
        CREATE TABLE IF NOT EXISTS admin
        (k TEXT PRIMARY KEY, v BLOB)
    "}
    .unwrap();
}
fn db_set(k: &str, v: Vec<u8>) {
    db! {"
        REPLACE INTO admin
        VALUES (?1, ?2)
    ", [k, v]}
    .unwrap();
}
fn _db_get(k: &str) -> Option<(Vec<u8>,)> {
    db! {"
        SELECT v FROM admin
        WHERE k = ?
    ", [k], ^(0)}
    .ok()
}

async fn post_handler(q: RawQuery, body: Bytes) {
    let q = q.0.unwrap();
    let k = q.split_once('=').unwrap().1;
    db_set(k, body.into());
}

pub fn service() -> Router {
    db_init();
    Router::new().route(
        "/admin",
        MethodRouter::new()
            .get(|| async { Html((include_page!("page.html") as [_; 1])[0]) })
            .post(post_handler)
            .layer(crate::auth::auth_layer()),
    )
}
