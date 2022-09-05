//! Auto submit JUST's health check-in form.
use crate::ticker::Ticker;
use crate::utils::fetch;
use crate::{care, db, include_page, strip_str};
use anyhow::Result;
use axum::extract::Form;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fmt::Write as _;
mod cryptojs;

fn db_init() {
    db!("CREATE TABLE health_list (id INTEGER PRIMARY KEY, pw TEXT, data TEXT)").ok();
    db!("CREATE TABLE health_log (time INTEGER, id INTEGER, ret TEXT)").ok();
}
fn db_list_set(id: u64, pw: String, data: String) {
    let sql = "REPLACE INTO health_list VALUES (?1, ?2, ?3)";
    db!(sql, [id, pw, data]).unwrap();
}
fn db_list_get() -> Vec<(u64, String, String)> {
    db!("SELECT * FROM health_list", [], (0, 1, 2)).unwrap()
}
fn db_log_insert(id: u64, ret: String) {
    let sql = "INSERT INTO health_log VALUES (strftime('%s','now'), ?1, ?2)";
    db!(sql, [id, ret]).unwrap();
}
fn db_log_get() -> Vec<(u64, u64, String)> {
    let sql = strip_str! {"
        SELECT * FROM health_log
        WHERE strftime('%s','now') - time <= 3600 * 24 * 5
        ORDER BY time DESC
    "};
    db!(sql, [], (0, 1, 2)).unwrap()
}
fn db_log_clean() {
    let sql = strip_str! {"
        DELETE FROM health_log
        WHERE strftime('%s','now') - time > 3600 * 24 * 7
    "};
    db!(sql).unwrap();
}

#[derive(Deserialize)]
struct Member {
    id: u64,
    pw: String,
    body: String,
}

async fn get_handler() -> impl IntoResponse {
    const PAGE: [&str; 2] = include_page!("page.html");
    let mut body = PAGE[0].to_string();
    for (time, id, ret) in db_log_get() {
        writeln!(&mut body, "{time} | {id} | {ret}").unwrap();
    }
    body += PAGE[1];
    Html(body)
}

async fn post_handler(Form(Member { id, pw, body }): Form<Member>) -> Redirect {
    db_list_set(id, pw, body);
    Redirect::to("/health")
}

#[allow(unreachable_code)]
pub fn service() -> Router {
    return Router::new();
    db_init();
    Router::new().route(
        "/health",
        MethodRouter::new()
            .post(post_handler)
            .layer(crate::auth::auth_layer()) // require auth only for post
            .get(get_handler),
    )
}

async fn check_in() -> Result<()> {
    // token = `sessionStorage.jwToken` on http://dc.just.edu.cn
    // search `formData/saveFormSubmitDataEncryption` in `umi.js`, dump post data
    // view result: http://dc.just.edu.cn/#/v2/formReportDetail/zGO2n4p7
    let list = db_list_get();
    for (id, token, body) in list {
        let uri = "http://dc.just.edu.cn/dfi/formData/saveFormSubmitDataEncryption";
        let request = hyper::Request::post(uri)
            .header("authentication", token)
            .body(body.into())?;
        let ret = String::from_utf8_lossy(&fetch(request).await?).replace('\n', "");
        let ret = askama_escape::escape(&ret, askama_escape::Html).to_string(); // XSS
        db_log_insert(id, ret);
    }
    Ok(())
}

static TICKER: Lazy<Ticker> = Lazy::new(|| Ticker::new_p8(&[(3, 10, 0), (5, 10, 0)]));
#[allow(unreachable_code)]
pub async fn tick() {
    return;
    if !TICKER.tick() {
        return;
    }
    care!(check_in().await).ok();
    db_log_clean();
}

/*
https://stackoverflow.com/questions/70212075/how-to-make-unsigned-right-shift-in-rust
https://stackoverflow.com/questions/51571066/what-are-the-exact-semantics-of-rusts-shift-operators
https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Unsigned_right_shift
*/
