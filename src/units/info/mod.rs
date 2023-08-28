//! Provide server info.

use crate::include_src;
use crate::utils::{fetch_text, str2req};
use axum::http::header::{CACHE_CONTROL, REFRESH};
use axum::response::{Html, IntoResponse};
use axum::routing::{MethodRouter, Router};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant, UNIX_EPOCH};

// use once_cell::sync::Lazy;
// static SYS_VER: Lazy<String> = Lazy::new(|| {
//     if cfg!(target_os = "linux") {
//         std::fs::read_to_string("/proc/version").unwrap()
//     } else {
//         "unknown".into()
//     }
// });

static START_TIME: AtomicI64 = AtomicI64::new(0);
static LAST_REFRESH: AtomicI64 = AtomicI64::new(0);
static LATENCY_BAIDU: AtomicI64 = AtomicI64::new(0);
static LATENCY_ALIYUN: AtomicI64 = AtomicI64::new(0);

async fn refresh(uri: &str, data: &AtomicI64) {
    let instant = Instant::now();
    let req = str2req(uri);
    match tokio::time::timeout(Duration::from_secs(3), fetch_text(req)).await {
        Ok(Ok(_)) => data.store(instant.elapsed().as_millis() as _, Ordering::SeqCst),
        Ok(Err(_)) => data.store(-9, Ordering::SeqCst), // network error
        Err(_) => data.store(-7, Ordering::SeqCst),     // timeout
    };
}

async fn get_handler() -> impl IntoResponse {
    const PAGE: [&str; 2] = include_src!("page.html");

    let now = UNIX_EPOCH.elapsed().unwrap().as_secs() as i64;

    let mut o = String::new();

    o += PAGE[0];

    o += concat!(
        env!("CARGO_PKG_NAME"),
        " version : ",
        env!("CARGO_PKG_VERSION"),
        "\nsqlite version : ",
    );
    o += rusqlite::version();
    o += "\n";

    // o += "os : ";
    // o += &SYS_VER;
    // o += "\n";

    o += "uptime : ";
    o += &(now - START_TIME.load(Ordering::SeqCst)).to_string();
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

    o += "server <-> baidu : ";
    match LATENCY_BAIDU.load(Ordering::SeqCst) {
        -9 => o += "network error\n",
        -7 => o += "timeout\n",
        v => {
            o += &v.to_string();
            o += " ms\n";
        }
    }

    o += "server <-> aliyun : ";
    match LATENCY_ALIYUN.load(Ordering::SeqCst) {
        -9 => o += "network error\n",
        -7 => o += "timeout\n",
        v => {
            o += &v.to_string();
            o += " ms\n";
        }
    }

    o += PAGE[1];

    ([(CACHE_CONTROL, "no-store")], Html(o))
}

pub fn service() -> Router {
    START_TIME.store(
        UNIX_EPOCH.elapsed().unwrap().as_secs() as _,
        Ordering::SeqCst,
    );
    Router::new()
        .route("/info", MethodRouter::new().get(get_handler))
        .route("/info/p", MethodRouter::new().get(|| async { "pong" })) // the "/ping" cause error?
}
