//! Auto submit JUST's health check-in form.
use crate::ticker::Ticker;
use crate::utils::{fetch, fetch_json, fetch_text, OptionResult};
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
    db!("CREATE TABLE health_list (id INTEGER PRIMARY KEY, password TEXT, data TEXT)").ok();
    db!("CREATE TABLE health_log (time INTEGER, id INTEGER, ret TEXT)").ok();
}
fn db_list_set(id: u64, password: String, data: String) {
    let sql = "REPLACE INTO health_list VALUES (?1, ?2, ?3)";
    db!(sql, [id, password, data]).unwrap();
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
    password: String,
    data: String,
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

async fn post_handler(Form(Member { id, password, data }): Form<Member>) -> Redirect {
    db_list_set(id, password, data);
    Redirect::to("/health")
}

async fn check_in() -> Result<()> {
    // token = `sessionStorage.jwToken` on http://dc.just.edu.cn
    // search `formData/saveFormSubmitDataEncryption` in `umi.js`, dump post data
    // view result: http://dc.just.edu.cn/#/v2/formReportDetail/zGO2n4p7
    const LOGIN_EXECUTION_VALUE: &str = include_str!("login_execution_value.txt");
    let form_wid = "a5e94ae0b0e04193bae67c86cfd6e223";
    for (id, password, data) in db_list_get() {
        let uri = "http://ids2.just.edu.cn/cas/login?service=http%3A%2F%2Fdc.just.edu.cn%2F%23%2F";
        let body = format!("username={id}&password={password}&execution={LOGIN_EXECUTION_VALUE}&_eventId=submit&encrypted=true&loginType=1&submit=%E7%99%BB+%E5%BD%95");
        let request = hyper::Request::post(uri)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body.into())?;
        let r = fetch(request).await?;
        dbg!(r.status());
        let r = r.headers().get("location").e()?.to_str()?;
        let ticket = r.split_once("ticket=").e()?.1.split_once('#').e()?.0;

        let uri = format!("http://dc.just.edu.cn/dfi/validateLogin?ticket={ticket}&service=http%3A%2F%2Fdc.just.edu.cn%2F%23%2F");
        let authentication = fetch_json(&uri, "/data/token").await?;

        let uri = format!("http://dc.just.edu.cn/dfi/formOpen/saveFormView?formWid={form_wid}");
        let request = hyper::Request::post(uri)
            .header("authentication", &authentication)
            .body(hyper::Body::empty())?;
        let submit_token = fetch_json(request, "/data/submitToken").await?;

        let body = format! {r#"{{"dataMap":{data},"formWid":"{form_wid}","submitToken":"{submit_token}"}}"#};
        let body = cryptojs::encrypt(body);
        let uri = "http://dc.just.edu.cn/dfi/formData/saveFormSubmitDataEncryption";
        let request = hyper::Request::post(uri)
            .header("authentication", &authentication)
            .body(body.into())?;
        let ret = fetch_text(request).await?.replace('\n', "");
        let ret = askama_escape::escape(&ret, askama_escape::Html).to_string(); // XSS

        db_log_insert(id, ret);
    }
    Ok(())
}

pub fn service() -> Router {
    db_init();
    Router::new().route(
        "/health",
        MethodRouter::new()
            .post(post_handler)
            .layer(crate::auth::auth_layer()) // require auth only for post
            .get(get_handler),
    )
}

static TICKER: Lazy<Ticker> = Lazy::new(|| Ticker::new_p8(&[(3, 10, 0), (5, 10, 0)]));
pub async fn tick() {
    // if !TICKER.tick() {
    //     return;
    // }
    dbg!(1);
    check_in().await.unwrap();
    // care!(check_in().await).ok();
    db_log_clean();
}
