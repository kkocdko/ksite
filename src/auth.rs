use crate::include_page;
use crate::units::admin::db_get;
use axum::body::Bytes;
use axum::http::header::{HeaderValue, COOKIE};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse};
use once_cell::sync::Lazy;
use std::io::Write;

static AUTH_COOKIE: Lazy<HeaderValue> = Lazy::new(|| {
    let mut inner = Vec::new();
    inner.extend(b"auth=");
    match db_get("auth_key") {
        Some((v,)) => inner.extend(v.as_slice()),
        None => write!(&mut inner, "{:x}", rand::random::<u128>()).unwrap(),
    }
    HeaderValue::from_maybe_shared(Bytes::from(inner)).unwrap()
});

pub fn auth_key() -> &'static str {
    // because this is a low frequency operation
    AUTH_COOKIE.to_str().unwrap().split_once('=').unwrap().1
}

pub async fn auth_layer<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    const AUTH_PAGE: &str = (include_page!("auth.html") as [_; 1])[0];
    match req
        .headers()
        .get_all(COOKIE)
        .into_iter()
        .any(|v| v == *AUTH_COOKIE)
    {
        true => next.run(req).await,
        false => (StatusCode::UNAUTHORIZED, Html(AUTH_PAGE)).into_response(),
    }
}

// https://docs.rs/axum/latest/axum/middleware/fn.from_fn.html
