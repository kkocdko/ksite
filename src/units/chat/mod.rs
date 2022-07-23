use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::Path;
use axum::response::Html;
use axum::routing::MethodRouter;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::broadcast::{self, Sender};

static ROOMS: Lazy<Mutex<HashMap<u32, Room>>> = Lazy::new(Default::default);

struct Room {
    user_count: u32,
    channel: Sender<String>,
}

async fn ws_handler(id: u32, ws: WebSocket) {
    let (ch_tx, mut ch_rx) = {
        let mut rooms = ROOMS.lock().unwrap();
        let room = rooms.entry(id).or_insert_with(|| Room {
            user_count: 0,
            channel: broadcast::channel(8).0,
        });
        room.user_count += 1;
        // broadcast::Sender::clone is based on Arc
        (room.channel.clone(), room.channel.subscribe())
    };

    let (mut ws_tx, mut ws_rx) = ws.split();

    // current -> server -> others
    let mut c2o = tokio::spawn(async move {
        while let Some(Ok(Message::Text(v))) = ws_rx.next().await {
            if ch_tx.send(v).is_err() {
                break;
            }
        }
    });

    // others -> server -> current
    let mut o2c = tokio::spawn(async move {
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

    let mut rooms = ROOMS.lock().unwrap();
    let room = rooms.get_mut(&id).unwrap();
    room.user_count -= 1;
    if room.user_count == 0 {
        rooms.remove(&id);
    }
}

pub fn service() -> Router {
    Router::new()
        .route(
            "/chat", // https://127.0.0.1:9304/chat#123
            MethodRouter::new().get(|| async { Html(include_str!("page.html")) }),
        )
        .route(
            "/chat/ws/:room",
            MethodRouter::new().get(|Path(id): Path<u32>, u: WebSocketUpgrade| {
                async move { u.on_upgrade(move |ws| ws_handler(id, ws)) }
                // if not returns the response produced by `on_upgrade`, the WebSocket
                // will receive a `ResetWithoutClosingHandshake` error
            }),
        )
}
