use crate::utils::{fetch_text, slot};
use axum::http::header::{CACHE_CONTROL, REFRESH};
use axum::response::{Html, IntoResponse};
use axum::routing::{MethodRouter, Router};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, UNIX_EPOCH};

static START_TIME: AtomicU64 = AtomicU64::new(0);
static LAST_GEN: AtomicU64 = AtomicU64::new(0);
static LATENCY_BAIDU: AtomicU64 = AtomicU64::new(0);
static LATENCY_ALIYUN: AtomicU64 = AtomicU64::new(0);

async fn refresh() {
    async fn latency(uri: &str, store: &AtomicU64) {
        let instant = Instant::now();
        let timeout = Duration::from_millis(3000);
        match tokio::time::timeout(timeout, fetch_text(uri)).await {
            Ok(Ok(_)) => store.store(instant.elapsed().as_millis() as _, Ordering::SeqCst),
            Ok(Err(_)) => store.store(9000, Ordering::SeqCst), // network error
            Err(_) => store.store(7000, Ordering::SeqCst),     // timeout
        };
    }
    tokio::join!(
        latency("http://baidu.com/404", &LATENCY_BAIDU),
        latency("http://aliyun.com/404", &LATENCY_ALIYUN),
    );
    LAST_GEN.store(UNIX_EPOCH.elapsed().unwrap().as_secs(), Ordering::SeqCst)
}

async fn get_handler() -> impl IntoResponse {
    const PAGE: [&str; 2] = slot(include_str!("page.html"));

    let now = UNIX_EPOCH.elapsed().unwrap().as_secs();

    let mut o = PAGE[0].to_string();

    o.push_str(concat!(
        env!("CARGO_PKG_NAME"),
        " v",
        env!("CARGO_PKG_VERSION"),
        "\n\n"
    ));

    o.push_str("Uptime : ");
    o.push_str(&(now - START_TIME.load(Ordering::Relaxed)).to_string());
    o.push_str(" s\n");

    if now - LAST_GEN.load(Ordering::SeqCst) > 5 {
        tokio::spawn(refresh());
        return ([(REFRESH, "1")], Html(o + PAGE[1]));
    }

    o.push_str("Server <-> Baidu : ");
    match LATENCY_BAIDU.load(Ordering::SeqCst) {
        9000 => o.push_str("network error\n"),
        7000 => o.push_str("timeout\n"),
        v => {
            o.push_str(&v.to_string());
            o.push_str(" ms\n");
        }
    }

    o.push_str("Server <-> Aliyun : ");
    match LATENCY_ALIYUN.load(Ordering::SeqCst) {
        9000 => o.push_str("network error\n"),
        7000 => o.push_str("timeout\n"),
        v => {
            o.push_str(&v.to_string());
            o.push_str(" ms\n");
        }
    }

    o.push_str(&PAGE[1]);

    ([(CACHE_CONTROL, "no-store")], Html(o))
}

pub fn service() -> Router {
    START_TIME.store(UNIX_EPOCH.elapsed().unwrap().as_secs(), Ordering::Relaxed);
    Router::new()
        .route("/info", MethodRouter::new().get(get_handler))
        .route("/info/p", MethodRouter::new().get(|| async { "pong" }))
}
