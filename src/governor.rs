use crate::include_src;
use crate::log;
use crate::units::admin::db_get;
use axum::body::Body;
use axum::body::Bytes;
use axum::extract::ConnectInfo;
use axum::extract::FromRequest;
use axum::http::header::{HeaderValue, COOKIE};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Response};
use once_cell::sync::Lazy;
use std::io::Write as _;
use std::net::SocketAddr;

// https://github.com/benwis/tower-governor
// https://github.com/antifuchs/governor
// https://juejin.cn/post/7056000911893594148
// https://github.com/antifuchs/governor/blob/master/governor/README.md

pub async fn governor_layer(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next<Body>,
) -> Response {
    addr.ip();
    log!("{:?}", addr == addr);
    log!("{:?}", addr);
    next.run(req).await
}
