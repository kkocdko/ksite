//! Admin console.

use crate::{db, include_page};
use axum::body::Bytes;
use axum::extract::RawQuery;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};

fn db_init() {
    db! {"
        CREATE TABLE IF NOT EXISTS admin
        (k TEXT PRIMARY KEY, v BLOB)
    "}
    .unwrap();
}
pub fn db_set(k: &str, v: Vec<u8>) {
    db! {"
        REPLACE INTO admin
        VALUES (?1, ?2)
    ", [k, v]}
    .unwrap();
}
pub fn db_get(k: &str) -> Option<(Vec<u8>,)> {
    db! {"
        SELECT v FROM admin
        WHERE k = ?
    ", [k], ^(0)}
    .ok()
}
pub fn db_del(k: &str) {
    db! {"
        DELETE FROM admin
        WHERE k = ?
    ", [k]}
    .unwrap();
}

async fn post_handler(q: RawQuery, body: Bytes) {
    match q.0.unwrap().as_str() {
        "noop" => {}
        "reset_auth_key" => {
            db_del("auth_key");
        }
        "backup_database" => {
            crate::database::backup();
        }
        k @ ("ssl_cert" | "ssl_key") => {
            db_set(k, body.into());
        }
        _ => {}
    }
}

pub fn service() -> Router {
    db_init();
    if db_get("auth_key").is_none() {
        db_set("auth_key", crate::auth::AUTH_KEY.to_owned().into_bytes());
    }
    Router::new().route(
        "/admin",
        MethodRouter::new()
            .get(|| async { Html((include_page!("page.html") as [_; 1])[0]) })
            .post(post_handler)
            .layer(crate::auth::auth_layer()),
    )
}
