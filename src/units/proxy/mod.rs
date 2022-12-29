//! Primitive proxy over http(s).
//!
//! 1. Inline Proxy Mode: view web page in browser directly, for simple usage.
//! 2. Shadowsocks Mode: simple shadowsocks proxy implement.
//! 3. UDP 53 Mode: through client's UDP 53 port.

use std::any::{Any, TypeId};

use crate::include_src;
use crate::utils::{fetch, read_body, OptionResult};
use axum::body::{Body, Bytes, HttpBody};
use axum::extract::FromRequest;
use axum::http::header::{HeaderMap, CONTENT_LENGTH};
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, EXPIRES, REFRESH};
use axum::http::{Request, Response, Uri};
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use std::mem;

// https://jimages.net/archives/269
// https://github.com/dizda/fast-socks5

async fn inline_proxy_handler(mut req: Request<Body>) -> Result<Response<Body>, &'static str> {
    let e = "invalid_request";
    let uri = req.uri_mut();
    let uri: Uri = uri.query().e().or(Err(e))?.try_into().or(Err(e))?;
    let path = uri.path();
    let mut is_html = path.ends_with(".htm")
        || path.ends_with(".html")
        || path.ends_with(".xhtml")
        || path.ends_with(".shtml");
    *req.uri_mut() = uri;
    let mut rep = fetch(req).await.or(Err("fetch_failed"))?;
    if let Some(v) = rep.headers().get(CONTENT_TYPE) {
        if let Ok(v) = v.to_str() {
            if v.contains("text/html;") || v.ends_with("text/html") {
                is_html = true;
            }
        }
    }
    if is_html {
        // rep.version()
        // view-source:https://127.0.0.1:9304/proxy/inline?https://www.bing.com/
        let mut body = read_body(mem::take(rep.body_mut())).await;
        dbg!(String::from_utf8_lossy(&body[body.len() - 32..]));
        const APPEND: &str = (include_src!("append.html") as [_; 1])[0];
        body.extend(APPEND.as_bytes());

        dbg!("is_html", APPEND);
        *rep.body_mut() = Body::from(body);
    }
    // TODO: CSP?
    Ok(rep)
}

pub fn service() -> Router {
    Router::new()
        .route(
            "/proxy", // home page
            MethodRouter::new().get(|| async { Html((include_src!("page.html") as [_; 1])[0]) }),
        )
        .route(
            "/proxy/inline", // inline proxy
            MethodRouter::new().fallback(inline_proxy_handler),
        )
        .route(
            "/proxy/sw.js", // inline proxy
            MethodRouter::new().get(|| async {
                (
                    [
                        (
                            CONTENT_TYPE,
                            HeaderValue::from_static("application/javascript"),
                        ),
                        (CACHE_CONTROL, HeaderValue::from_static("no-store")),
                        // (CACHE_CONTROL, HeaderValue::from_static("max-age=600")),
                    ],
                    (include_src!("sw.js") as [_; 1])[0],
                )
            }),
        )
}
