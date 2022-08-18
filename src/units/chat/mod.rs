//! Simple chat rooms, client-to-client encrypted.
use crate::utils::read_body;
use anyhow::Result;
use axum::extract::{Path, RawBody};
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

static ROOMS: Lazy<Mutex<HashMap<u32, Room>>> = Lazy::new(Default::default);

struct Room {
    user_count: u32,
    tx: Sender<String>,
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
    match room.tx.send(msg) {
        Ok(_) => "", // empty response body means succeeded
        Err(_) => "no receivers exist",
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
        factory: make_future,
    })
}

struct SseStream<T, F> {
    id: u32,
    fut: T,
    factory: F,
}

// Currently, Rust marks async block and async fn to `!Unpin`. However it may be safe, see:
// https://github.com/rust-lang/rust/issues/82187

#[cfg(feature = "sse-unsafe")]
async fn make_future(mut rx: Receiver<String>) -> (Option<String>, Receiver<String>) {
    (rx.recv().await.ok(), rx)
}
#[cfg(feature = "sse-unsafe")]
impl<T, F> Stream for SseStream<T, F>
where
    T: Future<Output = (Option<String>, Receiver<String>)>,
    F: Fn(Receiver<String>) -> T,
{
    type Item = Result<Event>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        unsafe {
            let this = self.get_unchecked_mut();
            let (value, rx) = ready!(Pin::new_unchecked(&mut this.fut).poll(cx));
            this.fut = (this.factory)(rx);
            Poll::Ready(value.map(|v| Ok(Event::default().data(v))))
        }
    }
}

// But I still provide a version without unsafe:

#[cfg(not(feature = "sse-unsafe"))]
fn make_future(
    mut rx: Receiver<String>,
) -> impl Future<Output = (Option<String>, Receiver<String>)> + Send + Unpin {
    Box::pin(async { (rx.recv().await.ok(), rx) })
}
#[cfg(not(feature = "sse-unsafe"))]
impl<T, F> Stream for SseStream<T, F>
where
    T: Future<Output = (Option<String>, Receiver<String>)> + Unpin,
    F: Fn(Receiver<String>) -> T + Unpin,
{
    type Item = Result<Event>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let (value, rx) = ready!(Pin::new(&mut this.fut).poll(cx));
        this.fut = (this.factory)(rx);
        Poll::Ready(value.map(|v| Ok(Event::default().data(v))))
    }
}

impl<F, T> Drop for SseStream<F, T> {
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
