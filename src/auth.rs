// Provide auth middleware.

use crate::include_src;
use crate::units::admin;
use crate::utils::LazyLock;
use axum::body::Body;
use axum::http::header::COOKIE;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Response};
use std::fmt::Write as _;

static AUTH_COOKIE: LazyLock<String> = LazyLock::new(|| {
    let mut inner = String::new();
    inner += "auth=";
    match admin::db::get("auth_key") {
        Some(v) => inner += std::str::from_utf8(&v).unwrap(),
        None => write!(&mut inner, "{:x}", rand::random::<u128>()).unwrap(),
    }
    inner
});

pub fn auth_key() -> &'static str {
    // because this is a low frequency operation
    AUTH_COOKIE.split_once('=').unwrap().1
}

pub async fn auth_layer(req: Request<Body>, next: Next) -> Response {
    const AUTH_PAGE: &str = (include_src!("auth.html") as [_; 1])[0];
    match req
        .headers()
        .get_all(COOKIE) // http2 allows multiple header entries with same name
        .into_iter()
        .any(|v| match v.to_str() {
            Ok(v) if v.find(&*AUTH_COOKIE).is_some() => true,
            _ => false,
        }) {
        true => next.run(req).await,
        false => (StatusCode::UNAUTHORIZED, Html(AUTH_PAGE)).into_response(),
    }
}

// https://docs.rs/axum/latest/axum/middleware/fn.from_fn.html
