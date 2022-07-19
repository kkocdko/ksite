use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Form, Path};
use axum::response::Html;
use axum::response::Redirect;
use axum::routing::MethodRouter;
use axum::Router;

use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast::{self, Sender};

static ROOMS: Lazy<Mutex<HashMap<u32, Arc<Room>>>> = Lazy::new(Default::default);

struct Room {
    id: u32,
    channel: Sender<String>,
}

impl Room {
    fn new(id: u32) -> Arc<Self> {
        let mut rooms = ROOMS.lock().unwrap();
        let this = rooms.entry(id).or_insert_with(|| {
            Arc::new(Self {
                id,
                channel: broadcast::channel(8).0,
            })
        });
        Arc::clone(this)
    }

    async fn ws_handler(self: Arc<Self>, ws: WebSocket) {
        let (mut ws_tx, mut ws_rx) = ws.split();

        // current -> server -> others
        let c2o_this = Arc::clone(&self);
        let mut c2o = tokio::spawn(async move {
            let ch_tx = &c2o_this.channel;
            while let Some(Ok(Message::Text(v))) = ws_rx.next().await {
                if ch_tx.send(v).is_err() {
                    break;
                }
            }
        });

        // others -> server -> current
        let o2c_this = Arc::clone(&self);
        let mut o2c = tokio::spawn(async move {
            let mut ch_rx = o2c_this.channel.subscribe();
            while let Ok(v) = ch_rx.recv().await {
                if ws_tx.send(Message::Text(v)).await.is_err() {
                    break;
                }
            }
        });

        // if any one of the tasks exit, abort another
        tokio::select! {
            _ = (&mut c2o) => o2c.abort(),
            _ = (&mut o2c) => c2o.abort(),
        };
    }
}

impl Drop for Room {
    fn drop(&mut self) {
        ROOMS
            .lock()
            .unwrap()
            .remove(&self.id)
            .expect("current Room is not contained in the ROOMS");
        println!("[chat] room {} droped", self.id);
    }
}

pub fn service() -> Router {
    Router::new()
        .route(
            "/chat", // http://127.0.0.1:9304/chat#123
            MethodRouter::new().get(|| async { Html(include_str!("page.html")) }),
        )
        .route(
            "/chat/ws/:room",
            MethodRouter::new().get(|Path(id): Path<u32>, u: WebSocketUpgrade| {
                let room = Room::new(id);
                async { u.on_upgrade(|ws| room.ws_handler(ws)) }
                // if not returns the response produced by `on_upgrade`, the WebSocket
                // will receive a `ResetWithoutClosingHandshake` error
            }),
        )
}
