use crate::DB;
use askama_escape as escape;
use axum::extract::{Form, Query};
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::MethodRouter;
use serde::Deserialize;

fn db_init() {
    let db = DB.lock().unwrap();
    let sql = "CREATE TABLE paste (id INTEGER PRIMARY KEY AUTOINCREMENT, data BLOB)";
    db.execute_batch(sql).ok(); // ignore error if already exists
}
fn db_insert(data: &str) -> usize {
    let db = DB.lock().unwrap();
    let sql = "INSERT INTO paste VALUES (NULL, ?)";
    db.prepare_cached(sql).unwrap().execute([data]).unwrap();
    let sql = "SELECT last_insert_rowid() FROM paste";
    let mut stmd = db.prepare_cached(sql).unwrap();
    stmd.query_row([], |r| r.get(0)).unwrap()
}
fn db_update(id: usize, data: &str) {
    let db = DB.lock().unwrap();
    let sql = "UPDATE paste SET data = ?1 WHERE id = ?2;";
    let mut stmd = db.prepare_cached(sql).unwrap();
    stmd.execute(rusqlite::params![data, id]).unwrap();
}
fn db_get(id: usize) -> Option<String> {
    let db = DB.lock().unwrap();
    let sql = "SELECT data FROM paste WHERE id = ?";
    let mut stmd = db.prepare_cached(sql).unwrap();
    stmd.query_row([id], |r| r.get(0)).ok()
}

#[derive(Deserialize)]
struct Params {
    id: Option<usize>,
}

#[derive(Deserialize)]
struct Submit {
    value: String,
}

async fn get_handler(Query(params): Query<Params>) -> impl IntoResponse {
    let value = { params.id }
        .and_then(|id| db_get(id))
        .map(|v| escape::escape(&v, escape::Html).to_string())
        .unwrap_or_else(|| "Hello world".to_string());
    Html(include_str!("page.html").replace("{value}", &value))
    // (Headers([("cache-control", "max-age=600")]), body)
}

async fn post_handler(
    (Query(params), Form(submit)): (Query<Params>, Form<Submit>),
) -> impl IntoResponse {
    let id = if let Some(v) = params.id {
        db_update(v, &submit.value);
        v
    } else {
        db_insert(&submit.value)
    };
    Redirect::to(format!("./?id={id}").parse().unwrap())
}

pub fn main() -> MethodRouter {
    db_init();
    MethodRouter::new().get(get_handler).post(post_handler)
}
