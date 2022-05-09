use crate::{db, ticker::Ticker};
use axum::extract::Form;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::MethodRouter;
use axum::Router;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::Mutex;

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
fn db_log_get() -> Vec<(String, u64, String)> {
    let sql = "
        SELECT datetime(time,'unixepoch','localtime'), id, ret FROM health_log
        WHERE strftime('%s','now') - time <= 3600 * 72
        ORDER BY time DESC
    ";
    db!(sql, [], (0, 1, 2)).unwrap()
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
        log += &format!("{time} | {id} | {ret}\n");
    }
    log = askama_escape::escape(&log, askama_escape::Html).to_string();
    const PAGE: &str = include_str!("page.html");
    const CRYPTJS: &str = include_str!("crypto-js.min.js");
    Html(PAGE.replace("{log}", &log) + "<script>" + CRYPTJS + "</script>")
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

static TICKER: Lazy<Mutex<Ticker>> =
    Lazy::new(|| Mutex::new(Ticker::new_p8(&[(3, 22, 0), (5, 22, 0)])));
pub async fn tick() {
    if !TICKER.lock().unwrap().tick() {
        return;
    }

    let list = db_list_get();
    let client = reqwest::Client::new();
    for member in list {
        let req = client
            .post("http://dc.just.edu.cn/dfi/formData/saveFormSubmitDataEncryption")
            .header("authentication", member.token)
            .body(member.body);
        let ret = req.send().await.unwrap().text().await.unwrap();
        db_log_insert(member.id, &ret);
    }
}
