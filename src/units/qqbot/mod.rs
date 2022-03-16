use crate::db::Database;
use askama_escape as escape;
use axum::extract::{Form, Json, Query};
use axum::response;
use axum::response::{Headers, Html, IntoResponse, Redirect};
use axum::routing::MethodRouter;
use axum::{routing::get, Router};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// impl IntoResponse for Option<String>{

// }

#[derive(Deserialize, Debug)]
struct Event {
    self_id: Option<i64>,
    message_type: Option<String>, //group
    raw_message: Option<String>,
}
async fn post_handler(Json(event): Json<Event>) -> impl IntoResponse {
    match &event.message_type {
        Some(v) if v == "group" => {}
        _ => return Default::default(),
    }
    let tigger_mark = format!("[CQ:at,qq={}]", &event.self_id.unwrap());
    match &event.raw_message {
        Some(v) if v.starts_with(&tigger_mark) => {}
        _ => return Default::default(),
    }
    let msg = event
        .raw_message
        .unwrap()
        .strip_prefix(&tigger_mark)
        .unwrap()
        .trim();
    // msg.split_whitespace().nth_back(0);

    // println!("{:?}", event);
    let reply = "hi";
    format!("{{ \"reply\": \"{reply}\"  }}")
}

pub fn service() -> MethodRouter {
    MethodRouter::new().post(post_handler)
}
