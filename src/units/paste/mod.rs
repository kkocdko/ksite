//! Online clipboard.

use crate::auth::auth_layer;
use crate::include_src;
use crate::utils::html_escape;
use axum::body::Bytes;
use axum::extract::Path;
use axum::middleware;
use axum::response::{Html, Redirect};
use axum::routing::{MethodRouter, Router};

pub mod db {
    use crate::database::DB;
    use crate::strip_str;
    pub fn init() {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            CREATE TABLE IF NOT EXISTS paste (id INTEGER PRIMARY KEY, data BLOB)
        "};
        let mut stmd = db.prepare(sql).unwrap();
        stmd.execute(()).unwrap();
    }
    pub fn set(id: u64, data: &[u8]) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            REPLACE INTO paste VALUES (?, ?)
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((id, data)).unwrap();
    }
    pub fn get(id: u64) -> Option<Vec<u8>> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT data FROM paste WHERE id = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.query_row((id,), |r| r.get(0)).ok()
    }
}

async fn get_handler(Path(id): Path<u64>) -> Html<Vec<u8>> {
    const PAGE: [&str; 2] = include_src!("page.html");
    let mut body = Vec::new();
    body.extend(PAGE[0].as_bytes());
    let h = tokio::task::spawn_blocking(move || db::get(id));
    match h.await.unwrap() {
        Some(v) => body.extend(v),
        None => body.extend(b"New entry"),
    };
    body.extend(PAGE[1].as_bytes());
    Html(body)
}

async fn post_handler(Path(id): Path<u64>, body: Bytes) {
    // https://github.com/djc/askama/blob/0.12.0/askama_escape/src/lib.rs
    // 要求客户端做好escape，这里只作校验
    tokio::task::spawn_blocking(move || db::set(id, &body));
}

pub fn service() -> Router {
    db::init();
    Router::new()
        .route("/paste", MethodRouter::new().get(Redirect::to("/paste/1")))
        .route(
            "/paste/:id",
            MethodRouter::new().get(get_handler).post(post_handler),
        )
    // .route_layer(middleware::from_fn(auth_layer))
}
/*
# void
./bombardier -c 128 http://127.0.0.1:9304/paste/1
before: Reqs/sec = 168975.11
after: Reqs/sec = 169773.62
# write
./bombardier -c 128 -m POST -b "$(seq 99)" http://127.0.0.1:9304/paste/1
before: Reqs/sec = 54917.50
after: Reqs/sec =
# read
./bombardier -c 128 http://127.0.0.1:9304/paste/1
before: Reqs/sec = 141475.76
after: Reqs/sec =
*/
