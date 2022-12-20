//! Primitive proxy over http(s).

/*

* Page mode: browser iframe <-> service worker <-> server. Save data mode.
* Raw mode: tcp and udp -> http -> browser -> client.

*/

use crate::include_page;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
// https://jimages.net/archives/269
// https://github.com/dizda/fast-socks5

pub fn service() -> Router {
   
    Router::new()
        .route(
            "/proxy",
            MethodRouter::new().get(|| async { Html((include_page!("page.html") as [_; 1])[0]) }),
        )
        .route(
            "/proxy/sw.js",
            MethodRouter::new().get(|| async { (include_page!("sw.js") as [_; 1])[0] }),
        )
}
