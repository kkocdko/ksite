//! Simple chat rooms, client-to-client encrypted.
use anyhow::Result;
use axum::extract::{Path, RawBody};
use axum::http::header::CACHE_CONTROL;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse};
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::mpsc;

// TODO: use BTreeMap instead? const fn Mutex::new()?
static ROOMS: Lazy<Mutex<HashMap<u32, Room>>> = Lazy::new(Default::default);

struct Room {
    user_count: u32,
    channel: Sender<String>,
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
    let (tx, rx) = mpsc::channel::<Result<Event>>(1);
    tokio::spawn(async move {
        while let Ok(v) = ch_rx.recv().await {
            if tx.send(Ok(Event::default().data(v))).await.is_err() {
                break;
            }
        }
    });
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Sse::new(stream).keep_alive(KeepAlive::default().interval(Duration::from_secs(5)))
}

async fn post_handler(Path(id): Path<u32>, RawBody(body): RawBody) -> impl IntoResponse {
    let body: Vec<u8> = hyper::body::to_bytes(body).await.unwrap().into();
    let rooms = ROOMS.lock().unwrap();
    let room = match rooms.get(&id) {
        Some(v) => v,
        None => return "room not exist",
    };
    room.channel
        .send(match String::from_utf8(body) {
            Ok(v) => v,
            Err(_) => return "illegal message",
        })
        .ok();
    ""
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
