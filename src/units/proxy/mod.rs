//! Primitive proxy over http(s).
//!
//! 1. Browser Mode: view web page with proxy directly, for simple usage.
//! 2. Shadowsocks Mode: simple shadowsocks proxy implement.
//! 3. UDP 53 Mode: through client's UDP 53 port.

use crate::include_src;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
// https://jimages.net/archives/269
// https://github.com/dizda/fast-socks5

pub fn service() -> Router {
    Router::new()
        .route(
            "/proxy",
            MethodRouter::new().get(|| async { Html((include_src!("page.html") as [_; 1])[0]) }),
        )
        .route(
            "/proxy/sw.js",
            MethodRouter::new().get(|| async { (include_src!("sw.js") as [_; 1])[0] }),
        )
}
