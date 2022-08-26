//! Admin console.
use crate::db;
use crate::utils::read_body;
use axum::extract::{RawBody, RawQuery};
use axum::response::Html;
use axum::routing::{MethodRouter, Router};

fn db_init() {
    db!("CREATE TABLE admin (k TEXT PRIMARY KEY, v BLOB)").ok();
}
fn db_set(k: &str, v: Vec<u8>) {
    db!("REPLACE INTO admin VALUES (?1, ?2)", [k, v]).unwrap();
}
fn _db_get(k: &str) -> Vec<u8> {
    let r = db!("SELECT v FROM admin WHERE k = ?", [k], |r| r.get(0));
    r.unwrap().pop().unwrap()
}

async fn post_handler(q: RawQuery, RawBody(body): RawBody) {
    let q = q.0.unwrap();
    let k = q.split_once('=').unwrap().1;
    let v = read_body(body).await;
    db_set(k, v);
}

pub fn service() -> Router {
    db_init();
    Router::new().route(
        "/admin",
        MethodRouter::new()
            .get(|| async { Html(include_str!("page.html")) })
            .post(post_handler)
            .layer(crate::auth::auth_layer()),
    )
}
