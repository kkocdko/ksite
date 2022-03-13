use axum::extract::{Form, Query};
use axum::response::Redirect;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post, MethodRouter},
    Json, Router,
};
use serde::Serialize;
use serde::{de, Deserialize, Deserializer};
use std::sync::Mutex;
use std::{borrow::Cow, net::SocketAddr};
use std::{fmt, str::FromStr};
// static template: String = String::from_utf8_lossy(include_bytes!("page.html")).to_string();
use rusty_leveldb::DB;
use tokio::sync::Mutex;
lazy_static! {
    pub static ref db: Mutex<DB> = {
        let path = env::current_exe().unwrap().with_file_name("db");
        Mutex::new(rusqlite::Connection::open(path).unwrap())
    };
}

#[derive(Deserialize, Debug)]
struct Params {
    id: Option<String>,
}
async fn get_handler(Query(params): Query<Params>) -> impl IntoResponse {
    let value = db
        .lock()
        .await
        .prepare_cached("SELECT value FROM paste WHERE id = (?)")
        .unwrap()
        .query_map([params.id.unwrap()], |row| row.get::<_, String>(0))
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
    let template = String::from_utf8_lossy(include_bytes!("page.html"));
    let response = template.replace("{content}", &value);
    Html(response)
}

#[derive(Deserialize, Debug)]
struct Submit {
    value: String,
}
async fn post_handler(Form(input): Form<Submit>) -> impl IntoResponse {
    db.lock()
        .await
        .prepare_cached("INSERT INTO paste VALUES (NULL, ?)")
        .unwrap()
        .execute([input.value])
        .unwrap();
    Redirect::to("/paste?id=fake_id".parse().unwrap())
}

// paste_content_{id:date_order}
pub async fn service() -> MethodRouter {
    let sql = "CREATE TABLE paste (id INTEGER PRIMARY KEY AUTOINCREMENT, value TEXT)";
    db.lock().await.execute(sql, []).ok();
    get(get_handler).post(post_handler)
}
