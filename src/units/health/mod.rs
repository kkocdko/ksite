//! Auto submit JUST's health check-in form.
//!
//! The prototype is https://github.com/kkocdko/user-scripts/blob/master/scripts/just-kit/health-check-in.js

use crate::auth::auth_layer;
use crate::ticker::Ticker;
use crate::utils::{fetch, fetch_json, fetch_text, log_escape, OptionResult};
use crate::{care, db, include_page};
use anyhow::Result;
use axum::extract::RawQuery;
use axum::http::header::{HeaderName, CONTENT_TYPE, USER_AGENT};
use axum::middleware;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use std::fmt::Write;
mod cryptojs;

fn db_init() {
    // this is a legacy module, so we keep the database struct not change
    db!("
        CREATE TABLE IF NOT EXISTS health_list
        (id INTEGER PRIMARY KEY, password TEXT, data TEXT);
        CREATE TABLE IF NOT EXISTS health_log
        (time INTEGER, id INTEGER, ret TEXT);
    ")
    .unwrap();
}
fn db_list_set(id: u64, password: &str, data: &str) {
    db!(
        "
        REPLACE INTO health_list
        VALUES (?, ?, ?)
        ",
        [id, password, data]
    )
    .unwrap();
}
fn db_list_get() -> Vec<(u64, String, String)> {
    db!(
        "
        SELECT * FROM health_list
        ",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    )
    .unwrap()
}
fn db_log_insert(id: u64, ret: String) {
    db!(
        "
        INSERT INTO health_log
        VALUES (strftime('%s', 'now'), ?1, ?2)
        ",
        [id, ret]
    )
    .unwrap();
}
fn db_log_get() -> Vec<(u64, u64, String)> {
    db!(
        "
        SELECT * FROM health_log
        ",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    )
    .unwrap()
}
fn db_log_clean() {
    db!("
        DELETE FROM health_log
        WHERE strftime('%s', 'now') - time > 3600 * 24 * 7
    ")
    .unwrap();
}

async fn get_handler() -> impl IntoResponse {
    const PAGE: [&str; 2] = include_page!("page.html");
    let mut body = String::new();
    body += PAGE[0];
    for (time, id, ret) in db_log_get().into_iter().rev() {
        writeln!(&mut body, "{time} | {id} | {ret}").unwrap();
    }
    body += PAGE[1];
    Html(body)
}

async fn post_handler(q: RawQuery, body: String) {
    let id = q.0.unwrap().parse().unwrap();
    let (password, data) = body.split_once('\n').unwrap();
    db_list_set(id, password, data);
}

async fn check_in() -> Result<()> {
    db_log_insert(0, "call check_in()".into());
    #[allow(clippy::declare_interior_mutable_const)]
    const AUTHENTICATION: HeaderName = HeaderName::from_static("authentication"); // not AUTHORIZATION
    const LOGIN_EXECUTION_VALUE: &str = include_str!("login_execution_value.txt");
    const FORM_WID: &str = "a5e94ae0b0e04193bae67c86cfd6e223";
    for (id, password, data) in db_list_get() {
        let uri = "http://ids2.just.edu.cn/cas/login?service=http%3A%2F%2Fdc.just.edu.cn%2F%23%2F";
        let body = format!("username={id}&password={password}&execution={LOGIN_EXECUTION_VALUE}&_eventId=submit&encrypted=true&loginType=1&submit=%E7%99%BB+%E5%BD%95");
        let request = hyper::Request::post(uri)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(USER_AGENT, "Chrome")
            .body(body.into())?;
        let r = fetch(request).await?;
        let r = r.headers().get("location").e()?.to_str()?;
        let ticket = r.split_once("ticket=").e()?.1.split_once('#').e()?.0;

        let uri = format!("http://dc.just.edu.cn/dfi/validateLogin?ticket={ticket}&service=http%3A%2F%2Fdc.just.edu.cn%2F%23%2F");
        let request = hyper::Request::get(uri).body(hyper::Body::empty())?;
        let authentication = fetch_json(request, "/data/token").await?;

        let uri = format!("http://dc.just.edu.cn/dfi/formOpen/saveFormView?formWid={FORM_WID}");
        let request = hyper::Request::post(uri)
            .header(AUTHENTICATION, &authentication)
            .body(hyper::Body::empty())?;
        let submit_token = fetch_json(request, "/data/submitToken").await?;

        let uri = "http://dc.just.edu.cn/dfi/formData/saveFormSubmitDataEncryption";
        let body = format! {r#"{{"dataMap":{data},"formWid":"{FORM_WID}","submitToken":"{submit_token}"}}"#};
        let body = cryptojs::encrypt4just(body);
        let request = hyper::Request::post(uri)
            .header(AUTHENTICATION, &authentication)
            .body(body.into())?;
        let ret = log_escape(&fetch_text(request).await?);

        db_log_insert(id, ret);
    }
    Ok(())
}

pub fn service() -> Router {
    db_init();
    Router::new()
        .route(
            "/health",
            MethodRouter::new()
                .post(post_handler)
                .route_layer(middleware::from_fn(auth_layer)) // require auth only for post
                .get(get_handler),
        )
        .route(
            "/health/trigger",
            MethodRouter::new()
                .get(|| async {
                    care!(check_in().await).ok();
                    Redirect::to("/health")
                })
                .route_layer(middleware::from_fn(auth_layer)),
        )
}

static TICKER: Lazy<Ticker> = Lazy::new(|| Ticker::new_p8(&[(6, 2, 0), (8, 2, 0)]));
pub async fn tick() {
    if !TICKER.tick() {
        return;
    }

    care!(check_in().await).ok();
    db_log_clean();
}
