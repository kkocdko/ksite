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
        WHERE strftime('%s','now') - time <= 3600 * 48
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
    for (timestamp, id, ret) in db_log_get() {
        log += &format!("{timestamp} | {id} | {ret}\n");
    }
    log = askama_escape::escape(&log, askama_escape::Html).to_string();
    Html(
        include_str!("page.html").replace("{log}", &log)
            + "<script>"
            + include_str!("crypto-js.min.js")
            + "</script>",
    )
}

async fn post_handler(Form(member): Form<Member>) -> impl IntoResponse {
    db_list_insert(&member);
    Redirect::to("/health")
}

pub fn service() -> Router {
    db_init();
    Router::new().route(
        "/health",
        MethodRouter::new().get(get_handler).post(post_handler),
    )
    // .layer(tower_http::compression::CompressionLayer::new().br(true))
    // .layer(tower_http::auth::RequireAuthorizationLayer::basic("", "password"))
}

static TICKER: Lazy<Mutex<Ticker>> = Lazy::new(|| {
    let patterns = [(3, 0, 0), (5, 0, 0)];
    Mutex::new(Ticker::new_p8(&patterns))
});
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

// https://docs.rs/tower-http/latest/tower_http/auth/
