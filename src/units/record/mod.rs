use crate::include_page;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::Path;
use axum::response::Html;
use axum::routing::MethodRouter;
use axum::Router;
use tokio::fs;

async fn ws_handler(mut ws: WebSocket) {
    while let Some(Ok(Message::Binary(v))) = ws.recv().await {
        println!("ws.recv() : {}", v.len());
    }
}

pub fn service() -> Router {
    Router::new()
        .route(
            "/record",
            MethodRouter::new().get(|| async { Html(include_page!("page.html")[0]) }),
        )
        .route(
            "/record/ws/:id",
            MethodRouter::new().get(|id: Path<String>, u: WebSocketUpgrade| {
                //
                async { u.on_upgrade(ws_handler) }
            }),
        )
        .layer(crate::auth::auth_layer())
}

// TODO: rename to `emergency`, supports SOS message broadcast etc.
