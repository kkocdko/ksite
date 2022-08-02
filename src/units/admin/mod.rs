use crate::db;
use axum::extract::{BodyStream, RawQuery};
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use futures_util::StreamExt;

fn db_init() {
    db!("CREATE TABLE admin (k TEXT UNIQUE, v BLOB)").ok();
}
fn db_set(k: &str, v: Vec<u8>) {
    db!("INSERT OR REPLACE INTO admin VALUES (?1, ?2)", [k, v]).unwrap();
}

async fn post_handler(q: RawQuery, mut body: BodyStream) {
    let q = q.0.unwrap();
    let k = q.as_str().split_once('=').unwrap().1;
    let mut v = Vec::new();
    while let Some(Ok(bytes)) = body.next().await {
        v.append(&mut bytes.to_vec());
    }
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
