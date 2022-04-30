use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{response::Html, routing::MethodRouter, Router};
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use tokio::sync::broadcast::{self, Sender};

static CHANNEL: Lazy<Sender<String>> = Lazy::new(|| broadcast::channel(16).0);

async fn ws_handler(ws: WebSocket) {
    let (mut sender, mut receiver) = ws.split();
    // current -> server -> others
    let mut c2o = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if CHANNEL.send(text).is_err() {
                break;
            }
        }
    });
    // others -> server -> current
    let mut o2c = tokio::spawn(async move {
        while let Ok(msg) = CHANNEL.subscribe().recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
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

pub fn service() -> Router {
    Router::new()
        .route(
            "/chat",
            MethodRouter::new().get(|| async { Html(include_str!("page.html")) }),
        )
        .route(
            "/chat/ws",
            MethodRouter::new().get(|u: WebSocketUpgrade| async { u.on_upgrade(ws_handler) }),
        )
}
