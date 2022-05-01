use crate::db;
use axum::extract::{Form, Path};
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::MethodRouter;
use axum::Router;
use serde::Deserialize;

fn db_init() {
    db!("CREATE TABLE paste (id INTEGER PRIMARY KEY AUTOINCREMENT, data BLOB)").ok();
}
fn db_insert(data: &str) -> u64 {
    db!("INSERT INTO paste VALUES (NULL, ?)", [data]).unwrap();
    db!("SELECT last_insert_rowid() FROM paste", [], (0)).unwrap()[0].0
}
fn db_update(id: u64, data: &str) {
    db!("UPDATE paste SET data = ?1 WHERE id = ?2;", [data, id]).unwrap();
}
fn db_get(id: u64) -> Option<String> {
    let result = db!("SELECT data FROM paste WHERE id = ?", [id], (0));
    result.ok()?.pop().map(|v| v.0)
}

async fn read(id: Option<u64>) -> impl IntoResponse {
    let value = id
        .and_then(db_get)
        .map(|v| askama_escape::escape(&v, askama_escape::Html).to_string())
        .unwrap_or_else(|| "New entry".to_string());
    Html(include_str!("page.html").replace("{value}", &value))
}

#[derive(Deserialize)]
struct Submit {
    value: String,
}

async fn insert(Form(submit): Form<Submit>) -> impl IntoResponse {
    let id = db_insert(&submit.value);
    Redirect::to(&format!("/paste/{id}"))
}

async fn update((Path(id), Form(submit)): (Path<u64>, Form<Submit>)) -> impl IntoResponse {
    db_update(id, &submit.value);
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
