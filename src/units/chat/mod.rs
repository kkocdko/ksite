//! Simple chat rooms, client-to-client encrypted.

use anyhow::Result;
use axum::extract::Path;
use axum::http::header::CACHE_CONTROL;
use axum::response::sse::{Event, Sse};
use axum::response::{Html, IntoResponse};
use axum::routing::{MethodRouter, Router};
use futures_core::{ready, Stream};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll};
use tokio::sync::broadcast::{self, Receiver, Sender};

struct Room {
    user_count: u32,
    tx: Sender<String>,
}

type SseStreamFuture = Pin<Box<dyn Future<Output = (Option<String>, Receiver<String>)> + Send>>;

fn make_future(mut rx: Receiver<String>) -> SseStreamFuture {
    Box::pin(async { (rx.recv().await.ok(), rx) })
}

struct SseStream {
    id: u32,
    fut: SseStreamFuture,
}

impl Stream for SseStream {
    type Item = Result<Event>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let (value, rx) = ready!(Pin::new(&mut this.fut).poll(cx));
        this.fut = make_future(rx);
        Poll::Ready(value.map(|v| Ok(Event::default().data(v))))
    }
}

impl Drop for SseStream {
    fn drop(&mut self) {
        let mut rooms = ROOMS.lock().unwrap();
        let room = rooms.get_mut(&self.id).unwrap();
        room.user_count -= 1;
        if room.user_count == 0 {
            rooms.remove(&self.id);
            // println!("> rooms.remove({})", self.id);
        }
    }
}

static ROOMS: Lazy<Mutex<HashMap<u32, Room>>> = Lazy::new(Default::default);

async fn post_handler(Path(id): Path<u32>, msg: String) -> impl IntoResponse {
    let rooms = ROOMS.lock().unwrap();
    let room = match rooms.get(&id) {
        Some(v) => v,
        None => return "room not exist",
    };
    match room.tx.send(msg) {
        Err(_) => "no receivers exist",
        Ok(_) => "", // empty response body means succeeded
    }
}

async fn sse_handler(Path(id): Path<u32>) -> impl IntoResponse {
    let mut rooms = ROOMS.lock().unwrap();
    let room = rooms.entry(id).or_insert_with(|| Room {
        user_count: 0,
        tx: broadcast::channel(16).0,
    });
    room.user_count += 1;
    let rx = room.tx.subscribe();
    Sse::new(SseStream {
        id,
        fut: make_future(rx),
    })
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
