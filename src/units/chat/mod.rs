//! Simple chat rooms, client-to-client encrypted.
use crate::utils::read_body;
use anyhow::Result;
use axum::extract::{Path, RawBody};
use axum::http::header::CACHE_CONTROL;
use axum::response::sse::{Event, Sse};
use axum::response::{Html, IntoResponse};
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::mpsc;

static ROOMS: Lazy<Mutex<HashMap<u32, Room>>> = Lazy::new(Default::default);

struct Room {
    user_count: u32,
    channel: Sender<String>,
}

async fn post_handler(Path(id): Path<u32>, body: RawBody) -> impl IntoResponse {
    let body = read_body(body.0).await;
    // limited to 512 KB
    if body.len() > 512 * 1024 {
        return "message too long";
    }
    let msg = match String::from_utf8(body) {
        Ok(v) => v,
        Err(_) => return "message is not valid utf8",
    };
    let rooms = ROOMS.lock().unwrap();
    let room = match rooms.get(&id) {
        Some(v) => v,
        None => return "room not exist",
    };
    match room.channel.send(msg) {
        Ok(_) => "", // empty response body means succeeded
        Err(_) => "no receivers exist",
    }
}

async fn sse_handler(Path(id): Path<u32>) -> impl IntoResponse {
    let mut ch_rx = {
        let mut rooms = ROOMS.lock().unwrap();
        let room = rooms.entry(id).or_insert_with(|| Room {
            user_count: 0,
            channel: broadcast::channel(16).0,
        });
        room.user_count += 1;
        room.channel.subscribe()
    };
    let (tx, rx) = mpsc::channel::<Result<Event>>(4);
    tokio::spawn(async move {
        let o2c = async {
            while let Ok(v) = ch_rx.recv().await {
                if tx.send(Ok(Event::default().data(v))).await.is_err() {
                    break;
                }
            }
        };
        tokio::select! {
            _ = o2c => {},
            _ = tx.closed() => {} // all receivers droped
        };
        let mut rooms = ROOMS.lock().unwrap();
        let room = rooms.get_mut(&id).unwrap();
        room.user_count -= 1;
        if room.user_count == 0 {
            rooms.remove(&id);
        }
        // the `o2c` will be canceled
    });
    Sse::new(tokio_stream::wrappers::ReceiverStream::new(rx))
}

pub fn service() -> Router {
    Router::new()
        .route(
            "/chat", // https://127.0.0.1:9304/chat#123
            MethodRouter::new().get(|| async {
                (
                    [(CACHE_CONTROL, "max-age=300")],
                    Html(include_str!("page.html")),
                )
            }),
        )
        .route("/chat/post/:room", MethodRouter::new().post(post_handler))
        .route("/chat/sse/:room", MethodRouter::new().get(sse_handler))
}
