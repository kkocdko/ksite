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
// use std::sync::Mutex;
use std::{borrow::Cow, net::SocketAddr};
use std::{fmt, str::FromStr};
// static template: String = String::from_utf8_lossy(include_bytes!("page.html")).to_string();
use lazy_static::lazy_static;
use std::env;
use std::fs;
use tokio::sync::Mutex;

lazy_static! {
    static ref DB: Mutex<rusty_leveldb::DB> = {
        let mut path = env::current_exe().unwrap().with_file_name("db");
        fs::create_dir_all(&path);
        path.push("paste");
        let mut db = rusty_leveldb::DB::open(path, rusty_leveldb::Options::default()).unwrap();
        if db.get(b"last_id").is_none() {
            db.put(b"last_id", b"0");
        }
        Mutex::new(db)
    };
}

#[derive(Deserialize, Debug)]
struct Params {
    id: Option<String>,
}
async fn get_handler(Query(mut params): Query<Params>) -> impl IntoResponse {
    let value = if let Some(id) = &mut params.id {
        id.insert_str(0, "value_");
        DB.lock().await.get(id.as_bytes()).unwrap_or_default()
    } else {
        Default::default()
    };
    let value = &*String::from_utf8_lossy(&value);
    let value = askama_escape::escape(value, askama_escape::Html).to_string();
    let template = String::from_utf8_lossy(include_bytes!("page.html"));
    let response = template.replace("{value}", &value);
    Html(response)
}

#[derive(Deserialize, Debug)]
struct Submit {
    value: String,
}
async fn post_handler(Form(input): Form<Submit>) -> impl IntoResponse {
    let mut id = DB.lock().await.get(b"last_id").unwrap();
    let mut id: u128 = String::from_utf8_lossy(&id).parse().unwrap();
    id += 1;
    DB.lock()
        .await
        .put(format!("value_{id}").as_bytes(), input.value.as_bytes())
        .unwrap();
    DB.lock()
        .await
        .put(b"last_id", id.to_string().as_bytes())
        .unwrap();
    DB.lock().await.flush();
    Redirect::to(format!("/paste?id={id}").parse().unwrap())
}

// paste_content_{id:date_order}
pub fn service() -> MethodRouter {
    get(get_handler).post(post_handler)
}
