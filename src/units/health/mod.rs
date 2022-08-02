use crate::ticker::Ticker;
use crate::utils::{fetch, slot};
use crate::{care, db};
use anyhow::Result;
use axum::extract::Form;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use serde::Deserialize;

fn db_init() {
    db!("CREATE TABLE health_list (id INTEGER, token TEXT, body TEXT)").ok();
    db!("CREATE TABLE health_log (time INTEGER, id INTEGER, ret TEXT)").ok();
}
fn db_list_insert(Member { id, token, body }: &Member) {
    let sql = "INSERT INTO health_list VALUES (?1, ?2, ?3)";
    db!(sql, [id, token, body]).unwrap();
}
fn db_list_get() -> Vec<Member> {
    let map = |r: (_, _, _)| Member {
        id: r.0,
        token: r.1,
        body: r.2,
    };
    let result = db!("SELECT * FROM health_list", [], (0, 1, 2));
    result.unwrap().into_iter().map(map).collect()
}
fn db_log_insert(id: u64, ret: &str) {
    let sql = "INSERT INTO health_log VALUES (strftime('%s','now'), ?1, ?2)";
    db!(sql, [id, ret]).unwrap();
}
fn db_log_get() -> Vec<(u64, u64, String)> {
    let sql = "
        SELECT time, id, ret FROM health_log
        WHERE strftime('%s','now') - time <= 3600 * 24 * 5
        ORDER BY time DESC
    ";
    db!(sql, [], (0, 1, 2)).unwrap()
}
fn db_log_clean() {
    let sql = "
        DELETE FROM health_log
        WHERE strftime('%s','now') - time > 3600 * 24 * 7
    ";
    db!(sql).unwrap();
}

#[derive(Deserialize, Debug)]
struct Member {
    id: u64,
    token: String,
    body: String,
}

async fn get_handler() -> impl IntoResponse {
    let mut log = String::new();
    for (time, id, ret) in db_log_get() {
        use std::fmt::Write as FmtWrite;
        writeln!(&mut log, "{time} | {id} | {ret}").unwrap();
    }
    log = askama_escape::escape(&log, askama_escape::Html).to_string();
    const PAGE: [&str; 3] = slot(include_str!("page.html"));
    const CRYPTOJS: &str = include_str!("crypto-js.min.js"); // cdnjs.cloudflare.com/ajax/libs/crypto-js/4.1.1/crypto-js.js
    Html([PAGE[0], &log, PAGE[1], CRYPTOJS, PAGE[2]].join(""))
}

async fn post_handler(Form(member): Form<Member>) -> impl IntoResponse {
    db_list_insert(&member);
    Redirect::to("/health")
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

async fn check_in() -> Result<()> {
    let list = db_list_get();
    for member in list {
        let uri = "http://dc.just.edu.cn/dfi/formData/saveFormSubmitDataEncryption";
        let request = hyper::Request::post(uri)
            .header("authentication", member.token)
            .body(member.body.into())?;
        let ret = String::from_utf8(fetch(request).await?)?;
        db_log_insert(member.id, &ret);
    }
    Ok(())
}

static TICKER: Lazy<Ticker> = Lazy::new(|| Ticker::new_p8(&[(3, 10, 0), (5, 10, 0)]));
pub async fn tick() {
    if !TICKER.tick() {
        return;
    }

    care!(check_in().await).ok();
    db_log_clean();
}
