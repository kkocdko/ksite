use crate::db::Database;
use askama_escape as escape;
use axum::extract::{Form, Query};
use axum::response::{Headers, Html, IntoResponse, Redirect};
use axum::routing::MethodRouter;
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    static ref DB: Database = Database::open("paste");
    static ref PAGE: &'static str = include_str!("page.html");
}

#[derive(Deserialize)]
struct Params {
    id: Option<String>,
}
async fn get_handler(Query(params): Query<Params>) -> impl IntoResponse {
    let value = { params.id }
        .and_then(|id| DB.get(&format!("value_{id}")))
        .map(|v| escape::escape(&v, escape::Html).to_string())
        .unwrap_or_else(|| "Hello world".to_string());
    let body = Html(PAGE.replace("{value}", &value));
    (Headers([("cache-control", "max-age=600")]), body)
}

#[derive(Deserialize)]
struct Submit {
    value: String,
}
async fn post_handler(Form(submit): Form<Submit>) -> impl IntoResponse {
    fn id_increase(id: &str) -> String {
        let mut ret = String::new();
        let mut remain = 1;
        for ch in id.chars() {
            let v = ch.to_digit(36).unwrap() + remain;
            remain = v / 36;
            ret.push(char::from_digit(v % 36, 36).unwrap());
        }
        if remain > 0 {
            ret.push(char::from_digit(remain, 36).unwrap());
        }
        ret
    }
    let id = DB.get("next_id").unwrap();
    DB.put(&format!("value_{id}"), &submit.value);
    DB.put("next_id", &id_increase(&id));
    Redirect::to(format!("/paste?id={id}").parse().unwrap())
}

pub fn main() -> MethodRouter {
    if DB.get("next_id").is_none() {
        DB.put("next_id", "1");
    }
    MethodRouter::new().get(get_handler).post(post_handler)
}
