//! Online clipboard.

use crate::auth::auth_layer;
use crate::include_src;
use crate::utils::html_escape;
use axum::extract::Path;
use axum::middleware;
use axum::response::{Html, Redirect};
use axum::routing::MethodRouter;
use axum::Router;

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
    pub fn set(id: u64, data: &str) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            REPLACE INTO paste VALUES (?, ?)
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((id, data.as_bytes())).unwrap();
    }
    pub fn get(id: u64) -> Option<String> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT data FROM paste WHERE id = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.query_row((id,), |r| Ok(String::from_utf8(r.get(0)?).unwrap()))
            .ok()
    }
}

async fn get_handler(Path(id): Path<u64>) -> Html<String> {
    const PAGE: [&str; 2] = include_src!("page.html");
    let mut body = String::new();
    body += PAGE[0];
    body += match &db::get(id) {
        Some(v) => v,
        None => "New entry",
    };
    body += PAGE[1];
    Html(body)
}

async fn post_handler(Path(id): Path<u64>, body: String) {
    db::set(id, &html_escape(&body));
}

pub fn service() -> Router {
    db::init();
    Router::new()
        .route(
            "/paste",
            MethodRouter::new().get(|| async { Redirect::to("/paste/1") }),
        )
        .route(
            "/paste/:id",
            MethodRouter::new().get(get_handler).post(post_handler),
        )
        .layer(middleware::from_fn(auth_layer))
}
