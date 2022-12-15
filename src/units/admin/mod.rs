//! Admin console.

use crate::auth::auth_layer;
use crate::{db, include_page};
use axum::body::Bytes;
use axum::extract::RawQuery;
use axum::middleware;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};

fn db_init() {
    db!("
        CREATE TABLE IF NOT EXISTS admin
        (k BLOB PRIMARY KEY, v BLOB)
    ")
    .unwrap();
}
pub fn db_set(k: &str, v: &[u8]) {
    db!(
        "
        REPLACE INTO admin
        VALUES (?1, ?2)
        ",
        [k.as_bytes(), v]
    )
    .unwrap();
}
pub fn db_get(k: &str) -> Option<Vec<u8>> {
    db!(
        "
        SELECT v FROM admin
        WHERE k = ?
        ",
        [k.as_bytes()],
        &|r| r.get(0)
    )
    .ok()
}
pub fn db_del(k: &str) {
    db!(
        "
        DELETE FROM admin
        WHERE k = ?
        ",
        [k.as_bytes()]
    )
    .unwrap();
}

async fn post_handler(q: RawQuery, body: Bytes) {
    let q = q.0.unwrap();
    let k = q.as_str();
    println!("received admin op {k}");
    match k {
        "noop" => {
            // do nothing
        }
        "reset_auth_key" => {
            db_del("auth_key");
            // need restart to take effect
        }
        "restart_process" => {
            std::process::exit(0);
        }
        "backup_database" => {
            crate::database::backup();
        }
        "ssl_cert" | "ssl_key" => {
            db_set(k, &body);
        }
        _ => {
            println!("unknown op");
        }
    }
}

pub fn service() -> Router {
    db_init();
    if db_get("auth_key").is_none() {
        db_set("auth_key", crate::auth::auth_key().as_bytes());
    }
    Router::new().route(
        "/admin",
        MethodRouter::new()
            .get(|| async { Html((include_page!("page.html") as [_; 1])[0]) })
            .post(post_handler)
            .layer(middleware::from_fn(auth_layer)),
    )
}
