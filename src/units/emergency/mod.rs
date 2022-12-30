//! Some emergency function like record video and audio as evidence, send SOS messages.

use crate::auth::auth_layer;
use crate::include_src;
use axum::body::Bytes;
use axum::extract::Path;
use axum::http::status::StatusCode;
use axum::middleware;
use axum::response::{Html, IntoResponse};
use axum::routing::MethodRouter;
use axum::Router;
use std::fmt::Write as _;
use std::io::Write as _;

mod db {
    use crate::db;

    pub const KIND_UNKNOWN: u8 = 0;
    pub const KIND_IMAGE: u8 = 1;
    pub const KIND_AUDIO: u8 = 2;
    pub const KIND_VIDEO: u8 = 3;

    pub fn init() {
        db!("
            CREATE TABLE IF NOT EXISTS emergency_chunks
            (id INTEGER PRIMARY KEY AUTOINCREMENT, timestamp INTEGER, kind INTEGER, data BLOB);
        ")
        .unwrap();
    }

    pub fn insert(data: &[u8], kind: u8) {
        db!(
            "
            INSERT INTO emergency_chunks
            VALUES (NULL, strftime('%s', 'now'), ?1, ?2)
            ",
            [data, kind]
        )
        .unwrap();
    }

    /// Returns `Vec<(id, timestamp, kind)>`
    pub fn list() -> Vec<(u64, u64, u8)> {
        db!(
            "
            SELECT id, timestamp, kind from emergency_chunks
            ",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        )
        .unwrap()
    }

    pub fn get(id: u64) -> Option<Vec<u8>> {
        db!(
            "
            SELECT data from emergency_chunks
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
            "/emergency",
            MethodRouter::new().get(|| async { Html((include_src!("page.html") as [_; 1])[0]) }),
        )
        .route(
            "/emergency/upload",
            MethodRouter::new().post(|body: Bytes| async move {
                match body[0] {
                    byte => {
                        dbg!(byte);
                        db::insert(&body, db::KIND_UNKNOWN)
                    }
                }
            }),
        )
        .route(
            "/emergency/download/:id",
            MethodRouter::new().get(|Path(id): Path<u64>| async move {
                match db::get(id) {
                    Some(v) => v.into_response(),
                    None => StatusCode::NOT_FOUND.into_response(),
                }
            }),
        )
        .route(
            "/emergency/list",
            MethodRouter::new().get(|| async {
                let mut ret = String::new();
                for (id, timestamp, kind) in db::list() {
                    writeln!(&mut ret, "{id} {timestamp} {kind}").unwrap();
                }
                ret
            }),
        )
        .layer(middleware::from_fn(auth_layer))
}
