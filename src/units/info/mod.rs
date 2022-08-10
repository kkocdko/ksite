//! Provide server info.
use crate::utils::{fetch_text, slot};
use axum::http::header::{CACHE_CONTROL, REFRESH};
use axum::response::{Html, IntoResponse};
use axum::routing::{MethodRouter, Router};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, UNIX_EPOCH};

static START_TIME: AtomicU64 = AtomicU64::new(0);
static LAST_REFRESH: AtomicU64 = AtomicU64::new(0);
static LATENCY_BAIDU: AtomicU64 = AtomicU64::new(0);
static LATENCY_ALIYUN: AtomicU64 = AtomicU64::new(0);

async fn refresh(uri: &str, data: &AtomicU64) {
    let instant = Instant::now();
    match tokio::time::timeout(Duration::from_secs(3), fetch_text(uri)).await {
        Ok(Ok(_)) => data.store(instant.elapsed().as_millis() as _, Ordering::SeqCst),
        Ok(Err(_)) => data.store(9000, Ordering::SeqCst), // network error
        Err(_) => data.store(7000, Ordering::SeqCst),     // timeout
    };
}

async fn get_handler() -> impl IntoResponse {
    const PAGE: [&str; 2] = slot(include_str!("page.html"));

    let now = UNIX_EPOCH.elapsed().unwrap().as_secs();

    let mut o = PAGE[0].to_string();

    o += concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));
    o += "\n\n";

    o += "Uptime : ";
    o += &(now - START_TIME.load(Ordering::Relaxed)).to_string();
    o += " s\n";

    if now - LAST_REFRESH.load(Ordering::SeqCst) > 5 {
        tokio::spawn(async move {
            LAST_REFRESH.store(now, Ordering::SeqCst);
            tokio::join!(
                refresh("http://baidu.com/404", &LATENCY_BAIDU),
                refresh("http://aliyun.com/404", &LATENCY_ALIYUN),
            );
        });
        return ([(REFRESH, "1")], Html(o + PAGE[1]));
    }

    o += "Server <-> Baidu : ";
    match LATENCY_BAIDU.load(Ordering::SeqCst) {
        9000 => o += "network error\n",
        7000 => o += "timeout\n",
        v => {
            o += &v.to_string();
            o += " ms\n";
        }
    }

    o += "Server <-> Aliyun : ";
    match LATENCY_ALIYUN.load(Ordering::SeqCst) {
        9000 => o += "network error\n",
        7000 => o += "timeout\n",
        v => {
            o += &v.to_string();
            o += " ms\n";
        }
    }

    o += PAGE[1];

    ([(CACHE_CONTROL, "no-store")], Html(o))
}

pub fn service() -> Router {
    START_TIME.store(UNIX_EPOCH.elapsed().unwrap().as_secs(), Ordering::Relaxed);
    Router::new()
        .route("/info", MethodRouter::new().get(get_handler))
        .route("/info/p", MethodRouter::new().get(|| async { "pong" }))
}
