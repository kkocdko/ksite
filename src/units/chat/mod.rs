//! Simple chat rooms, client-to-client encrypted.

use crate::include_src;
use anyhow::Result;
use axum::extract::Path;
use axum::http::header::CACHE_CONTROL;
use axum::http::HeaderValue;
use axum::response::sse::{Event as SseEvent, Sse};
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use futures_core::Stream;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::ready;
use std::task::{Context, Poll};
use tokio::sync::broadcast::{self, Receiver, Sender};

type BroadcastSseFut = Pin<Box<dyn Future<Output = (Option<String>, Receiver<String>)> + Send>>;

struct BroadcastSse(BroadcastSseFut, Box<dyn Fn() + Send>);

impl BroadcastSse {
    fn new(rx: Receiver<String>, on_drop: Box<dyn Fn() + Send>) -> Self {
        Self(Self::make_fut(rx), on_drop)
    }

    fn make_fut(mut rx: Receiver<String>) -> BroadcastSseFut {
        Box::pin(async { (rx.recv().await.ok(), rx) })
    }
}

impl Stream for BroadcastSse {
    type Item = Result<SseEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let (value, rx) = ready!(Pin::new(&mut this.0).poll(cx));
        this.0 = Self::make_fut(rx);
        Poll::Ready(value.map(|v| Ok(SseEvent::default().data(v))))
    }
}

impl Drop for BroadcastSse {
    fn drop(&mut self) {
        self.1();
    }
}

struct Room {
    user_count: u32,
    tx: Sender<String>,
}

#[derive(Default)]
pub struct ChatServer {
    rooms: Arc<Mutex<HashMap<u32, Room>>>,
}

impl ChatServer {
    pub fn post_router(&self) -> MethodRouter {
        let rooms_mutex = self.rooms.clone();
        MethodRouter::new().post(|Path(id): Path<u32>, msg: String| async move {
            let rooms = rooms_mutex.lock().unwrap();
            let room = match rooms.get(&id) {
                Some(v) => v,
                None => return "room not exist",
            };
            match room.tx.send(msg) {
                Err(_) => "no receivers exist",
                Ok(_) => "", // empty response body means succeeded
            }
        })
    }

    pub fn sse_router(&self) -> MethodRouter {
        let rooms_mutex = self.rooms.clone();
        MethodRouter::new().get(|Path(id): Path<u32>| async move {
            let mut rooms = rooms_mutex.lock().unwrap();
            let room = rooms.entry(id).or_insert_with(|| Room {
                user_count: 0,
                tx: broadcast::channel(16).0,
            });
            room.user_count += 1;
            let rx = room.tx.subscribe();
            drop(rooms);
            Sse::new(BroadcastSse::new(
                rx,
                Box::new(move || {
                    let mut rooms = rooms_mutex.lock().unwrap();
                    let room = rooms.get_mut(&id).unwrap();
                    room.user_count -= 1;
                    if room.user_count == 0 {
                        rooms.remove(&id);
                        // log!("> rooms.remove({})", self.id);
                    }
                }),
            ))
        })
    }
}

static CHAT_SERVER: Lazy<ChatServer> = Lazy::new(Default::default);

pub fn service() -> Router {
    Router::new()
        .route(
            "/chat", // https://127.0.0.1:9304/chat#123
            MethodRouter::new().get((
                [(CACHE_CONTROL, HeaderValue::from_static("max-age=300"))],
                Html((include_src!("page.html") as [_; 1])[0]),
            )),
        )
        .route("/chat/post/:room", CHAT_SERVER.post_router())
        .route("/chat/sse/:room", CHAT_SERVER.sse_router())
}
