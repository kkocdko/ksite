//! Some emergency function like record video and audio as evidence, send SOS messages.

use crate::auth::auth_layer;
use crate::include_src;
use axum::body::{Body, Bytes};
use axum::extract::{FromRequest, Path};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use axum::middleware;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{MethodRouter, Router};
use std::fmt::Write as _;

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

pub fn service() -> Router {
    db::init();
    Router::new()
        .route(
            "/media/upload",
            MethodRouter::new().post(|mut req: Request<Body>| async move {
                let mime = req.headers_mut().remove(CONTENT_TYPE).unwrap();
                let body = Bytes::from_request(req, &()).await.unwrap();
                db::insert(mime.to_str().unwrap(), &body);
                // dbg!(&mime);
            }),
        )
        .route(
            "/media/download/:id",
            MethodRouter::new().get(|Path(id): Path<u64>| async move {
                match db::get(id) {
                    Some(v) => v.into_response(),
                    None => StatusCode::NOT_FOUND.into_response(),
                }
            }),
        )
        .route(
            "/media/list",
            MethodRouter::new().get(|| async {
                let mut ret = String::new();
                for (id, timestamp, mime) in db::list() {
                    writeln!(&mut ret, "{id} {timestamp} {mime}").unwrap();
                }
                ret
            }),
        )
        .route(
            "/media/auth",
            MethodRouter::new().get(|| async { Redirect::to("/media") }),
        )
        .layer(middleware::from_fn(auth_layer))
        .route(
            "/media",
            MethodRouter::new().get(|| async { Html((include_src!("page.html") as [_; 1])[0]) }),
        )
}
