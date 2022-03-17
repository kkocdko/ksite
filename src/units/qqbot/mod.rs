use axum::extract::Json;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::Mutex;

lazy_static! {
    static ref REPLIES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::from([
        ("呜".into(), "呜".into()),
        ("你说对吧".into(), "啊对对对".into()),
        ("运行平台".into(), "ksite / axum / mirai-go".into()),
    ]));
}

async fn fetch_text(url: &str) -> String {
    reqwest::get(url).await.unwrap().text().await.unwrap()
}
async fn fetch_json(url: &str) -> serde_json::Value {
    serde_json::to_value(&fetch_text(url).await).unwrap()
}

#[derive(Deserialize)]
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

    let msg = event.raw_message.as_ref().unwrap();
    let msg = msg.strip_prefix(&tigger_mark).unwrap();
    let mut msg = msg.trim().split_whitespace();
    let reply = |v| format!(r#"{{ "reply": "[BOT] {v}" }}"#);
    match msg.next().unwrap() {
        "乌克兰" | "俄罗斯" | "俄乌" => reply("嘘！"),
        "吟诗" => {
            let r = fetch_json("https://v1.jinrishici.com/all.json").await;
            reply(r["content"].as_str().unwrap())
        }
        "比特币" | "BTC" => {
            let r = fetch_json("https://chain.so/api/v2/get_info/BTC").await;
            let price = r["data"]["price"].as_str().unwrap().trim_matches('0');
            reply(&format!("比特币当前价格 {price} 美元"))
        }
        "垃圾分类" => {
            let i = msg.next().unwrap();
            let r = fetch_json(&format!("https://api.muxiaoguo.cn/api/lajifl?m={i}")).await;
            reply(&match r["data"]["type"].as_str() {
                Some(v) => format!("{i} {v}"),
                None => format!("鬼知道 {i} 是什么垃圾呢"),
            })
        }
        "聊天" => {
            let i = msg.next().unwrap();
            let url = format!("http://api.api.kingapi.cloud/api/xiaoai.php?msg={i}");
            reply(fetch_text(&url).await.split('\n').nth(2).unwrap())
        }
        "设置回复" => {
            let mut replies = REPLIES.lock().await;
            replies.insert(msg.next().unwrap().into(), msg.next().unwrap().into());
            reply("记住啦")
        }
        k if REPLIES.lock().await.contains_key(k) => reply(REPLIES.lock().await.get(k).unwrap()),
        _ => reply("未知指令"),
    }
}

pub fn service() -> MethodRouter {
    MethodRouter::new().post(post_handler)
}
