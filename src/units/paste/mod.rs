//! Online clipboard.
use crate::{db, include_page};
use axum::extract::{Form, Path};
use axum::response::{Html, Redirect};
use axum::routing::MethodRouter;
use axum::Router;
use serde::Deserialize;

fn db_init() {
    db!("CREATE TABLE paste (id INTEGER PRIMARY KEY AUTOINCREMENT, data BLOB)").ok();
}
fn db_insert(data: &str) -> u64 {
    db!("INSERT INTO paste VALUES (NULL, ?)", [data], &).unwrap() as _
}
fn db_update(id: u64, data: &str) {
    db!("UPDATE paste SET data = ?1 WHERE id = ?2;", [data, id]).unwrap();
}
fn db_get(id: u64) -> Option<String> {
    db!("SELECT data FROM paste WHERE id = ?", [id], ^|r| r.get(0)).ok()
}

fn escape(v: &str) -> String {
    askama_escape::escape(v, askama_escape::Html).to_string()
}

async fn read(id: Option<u64>) -> Html<String> {
    let value = id.and_then(db_get);
    let value = value.unwrap_or_else(|| "New entry".to_string());
    const PAGE: [&str; 2] = include_page!("page.html");
    Html([PAGE[0], &value, PAGE[1]].join(""))
}

#[derive(Deserialize)]
struct Data {
    value: String,
}

async fn insert(form: Form<Data>) -> Redirect {
    let id = db_insert(&escape(&form.value));
    Redirect::to(&format!("/paste/{id}"))
}

async fn update((Path(id), form): (Path<u64>, Form<Data>)) -> Redirect {
    db_update(id, &escape(&form.value));
    Redirect::to(&format!("/paste/{id}"))
}

pub fn service() -> Router {
    db_init();
    Router::new()
        .route(
            "/paste",
            MethodRouter::new().get(|| read(None)).post(insert),
        )
        .route(
            "/paste/:id",
            MethodRouter::new()
                .get(|Path(id): Path<u64>| read(Some(id)))
                .post(update),
        )
}
