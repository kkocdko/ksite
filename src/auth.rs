// Provide auth middleware.

use crate::include_src;
use crate::units::admin;
use crate::utils::{rand_id, LazyLock};
use axum::body::Body;
use axum::http::header::COOKIE;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Response};

static AUTH_COOKIE: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut inner = Vec::new();
    inner.extend(b"auth=");
    match admin::db::get("auth_key") {
        Some(v) => inner.extend(v),
        None => inner.extend(rand_id(&[32])),
    }
    inner
});

pub fn auth_key() -> &'static str {
    // because this is a low frequency operation
    std::str::from_utf8(&AUTH_COOKIE[b"auth=".len()..]).unwrap()
}

pub async fn auth_layer(req: Request<Body>, next: Next) -> Response {
    const AUTH_PAGE: &str = (include_src!("auth.html") as [_; 1])[0];
    match req
        .headers()
        .get_all(COOKIE) // http2 allows multiple header entries with same name
        .into_iter()
        .any(|v| cookie_match(v.as_bytes(), &AUTH_COOKIE))
    {
        true => next.run(req).await,
        false => (StatusCode::UNAUTHORIZED, Html(AUTH_PAGE)).into_response(),
    }
}

// https://docs.rs/axum/latest/axum/middleware/fn.from_fn.html

fn cookie_match(mut v: &[u8], needle: &[u8]) -> bool {
    loop {
        if v.starts_with(needle) {
            return true;
        }
        loop {
            match v.first() {
                None => return false,
                Some(&c) => {
                    v = &v[1..];
                    if c == b';' && v.first().is_some() {
                        v = &v[1..];
                        break;
                    }
                }
            };
        }
    }
}

fn cookie_match_slow(v: &[u8], needle: &[u8]) -> bool {
    for mut part in v.split(|&c| c == b';') {
        if part.first() == Some(&b' ') {
            part = &part[1..];
        }
        if part.starts_with(needle) {
            return true;
        }
    }
    false
}

#[allow(dead_code)]
fn test_cookie_match() {
    let cases = &[
        "auth=xxx",
        "  auth=xxx  ",
        "; auth=xxx  ",
        " ;auth=xxx  ",
        " ; auth=xxx  ",
        ";;auth=xxx  ",
        "some;auth=xxx",
        "some ;auth=xxx",
        "some; auth=xxx",
        "some ; auth=xxx",
    ];

    for v in cases {
        println!(
            "{:?} {:?} {:?}",
            cookie_match_slow(v.as_bytes(), b"auth=xxx"),
            cookie_match(v.as_bytes(), b"auth=xxx"),
            v
        );
    }
}
