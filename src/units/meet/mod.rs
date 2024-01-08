//! WebRTC meeting, supports real-time cloud record.

use super::chat::ChatServer;
use crate::include_src;
use axum::http::header::{HeaderValue, CACHE_CONTROL};
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use crate::utils::LazyLock as Lazy;

static CHAT_SERVER: Lazy<ChatServer> = Lazy::new(Default::default);

pub fn service() -> Router {
    // db::init();
    // ~/misc/apps/miniserve --header Cache-Control:no-store -p 9453 $(dirname $0)
    Router::new()
        .route(
            "/meet",
            MethodRouter::new().get((
                #[cfg(debug_assertions)]
                [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
                #[cfg(not(debug_assertions))]
                [(CACHE_CONTROL, HeaderValue::from_static("max-age=300"))],
                Html((include_src!("page.html") as [_; 1])[0]),
            )),
        )
        .route("/meet/post/:room", CHAT_SERVER.post_router())
        .route("/meet/sse/:room", CHAT_SERVER.sse_router())
}

/*
use crate::auth::auth_layer;
use axum::body::{Body, Bytes};
use axum::extract::{FromRequest, Path};
use axum::http::{Request, StatusCode};
use axum::middleware;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::broadcast;
mod db {
    use crate::db;

    pub fn init() {
        db!("
            CREATE TABLE IF NOT EXISTS media_chunks
            (time INTEGER, mime BLOB, data BLOB);
        ")
        .unwrap();
    }

    pub fn insert(mime: &str, data: &[u8]) {
        db!(
            "
            INSERT INTO media_chunks
            VALUES (NULL, strftime('%s', 'now'), ?1, ?2)
            ",
            [mime.as_bytes(), data]
        )
        .unwrap();
    }

    /// Returns `Vec<(id, time, mime)>`
    pub fn list() -> Vec<(u64, u64, String)> {
        db!(
            "
            SELECT rowid, time, mime from media_chunks
            ",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, String::from_utf8(r.get(2)?).unwrap()))
        )
        .unwrap()
    }

    pub fn get(id: u64) -> Option<Vec<u8>> {
        db!(
            "
            SELECT data from media_chunks
            WHERE id = ?
            ",
            [id],
            *|r| r.get(0)
        )
        .ok()
    }
}
.route(
    "/media/upload",
    MethodRouter::new().post(|mut req: Request<Body>| async move {
        let mime = req.headers_mut().remove(CONTENT_TYPE).unwrap();
        let body = Bytes::from_request(req, &()).await.unwrap();
        db::insert(mime.to_str().unwrap(), &body);
    }),
)
.route(
    "/meet/download/:id",
    MethodRouter::new().get(|Path(id): Path<u64>| async move {
        match db::get(id) {
            Some(v) => v.into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }),
)
.route(
    "/meet/list",
    MethodRouter::new().get(|| async {
        let mut ret = String::new();
        for (id, timestamp, mime) in db::list() {
            writeln!(&mut ret, "{id} {timestamp} {mime}").unwrap();
        }
        ret
    }),
)
.layer(middleware::from_fn(auth_layer))
*/
