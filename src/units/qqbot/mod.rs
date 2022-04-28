use axum::extract::Json;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;
use axum::Router;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::Mutex;

static REPLIES: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    Mutex::new(HashMap::from([
        ("呜".into(), "呜".into()),
        ("你说对吧".into(), "啊对对对".into()),
        ("运行平台".into(), "ksite / axum / mirai-go".into()),
    ]))
});

async fn fetch_text(url: &str) -> String {
    reqwest::get(url).await.unwrap().text().await.unwrap()
}
async fn fetch_json(url: &str) -> serde_json::Value {
    serde_json::from_str(&fetch_text(url).await).unwrap()
}

fn elapse(time: f64) -> f64 {
    // javascript: new Date("2001.01.01 06:00").getTime()
    let epoch = SystemTime::UNIX_EPOCH;
    let now = SystemTime::now().duration_since(epoch).unwrap().as_millis() as f64;
    (now - time) / 864e5
}

#[derive(Deserialize)]
struct Event {
    self_id: Option<i64>,
    raw_message: Option<String>,
}
async fn post_handler(Json(event): Json<Event>) -> impl IntoResponse {
    let trigger = format!("[CQ:at,qq={}]", &event.self_id.unwrap_or_default());
    let msg = match &event.raw_message {
        Some(v) if v.starts_with(&trigger) => v.strip_prefix(&trigger).unwrap(),
        _ => return Default::default(),
    };
    let msg: Vec<&str> = msg.trim().split_whitespace().collect();
    let reply = |v| format!(r#"{{ "reply": "[BOT] {v}" }}"#);
    match msg[0] {
        "乌克兰" | "俄罗斯" | "俄乌" => reply("嘘！"),
        "kk单身多久了" => reply(&format!("kk已连续单身 {:.3} 天了", elapse(10485432e5))),
        "暑假倒计时" => reply(&format!("距 2022 暑假仅 {:.3} 天", -elapse(16574688e5))),
        "高考倒计时" => reply(&format!("距 2022 高考仅 {:.3} 天", -elapse(16545636e5))),
        "驶向深蓝" => {
            let url = "https://api.lovelive.tools/api/SweetNothings?genderType=M";
            reply(&fetch_text(url).await)
        }
        "吟诗" => {
            let url = "https://v1.jinrishici.com/all.json";
            reply(fetch_json(url).await["content"].as_str().unwrap())
        }
        "新闻" => {
            let r = fetch_text("https://m.cnbeta.com/wap").await;
            let n = elapse(0.0) as usize % 20 + 3;
            reply(r.split("htm\">").nth(n).unwrap().split_once('<').unwrap().0)
        }
        "比特币" | "BTC" => {
            let r = fetch_json("https://chain.so/api/v2/get_info/BTC").await;
            let price = r["data"]["price"].as_str().unwrap().trim_matches('0');
            reply(&format!("比特币当前价格 {price} 美元"))
        }
        "垃圾分类" => {
            let i = msg[1];
            let r = fetch_json(&format!("https://api.muxiaoguo.cn/api/lajifl?m={i}")).await;
            reply(&match r["data"]["type"].as_str() {
                Some(v) => format!("{i} {v}"),
                None => format!("鬼知道 {i} 是什么垃圾呢"),
            })
        }
        "聊天" => {
            let url = format!("https://api.ownthink.com/bot?spoken={}", msg[1]);
            let r = fetch_json(&url).await;
            reply(r["data"]["info"]["text"].as_str().unwrap())
        }
        "设置回复" => {
            REPLIES.lock().await.insert(msg[1].into(), msg[2].into());
            reply("记住啦")
        }
        k if REPLIES.lock().await.contains_key(k) => reply(&REPLIES.lock().await[k]),
        _ => reply("未知指令"),
    }
}

pub fn service() -> Router {
    Router::new().route("/qqbot", MethodRouter::new().post(post_handler))
}

pub async fn tick() {}
