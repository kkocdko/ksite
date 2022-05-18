use crate::db;
use crate::ticker::Ticker;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::routing::MethodRouter;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::Mutex;

fn db_init() {
    db!("CREATE TABLE qqbot_groups (group_id INTEGER)").ok();
}
fn db_groups_get() -> Vec<i64> {
    let result = db!("SELECT * FROM qqbot_groups", [], (0));
    result.unwrap().into_iter().map(|r| r.0).collect()
}
fn db_groups_insert(group_id: i64) {
    db!("INSERT INTO qqbot_groups VALUES (?)", [group_id]).unwrap();
}

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

fn op_send_group_msg(group_id: i64, msg: &str) -> String {
    serde_json::json!({
        "action": "send_group_msg",
        "params": { "group_id": group_id, "message": msg }
    })
    .to_string()
}

static REPLIES: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    Mutex::new(HashMap::from([
        ("呜".into(), "呜".into()),
        ("你说对吧".into(), "啊对对对".into()),
        ("运行平台".into(), "ksite / axum / mirai-go".into()),
    ]))
});

async fn event_handler(event: serde_json::Value) -> Option<String> {
    let self_id = event.get("self_id")?.as_i64()?;
    let group_id = event.get("group_id")?.as_i64()?;
    let raw_message = event.get("raw_message")?.as_str()?;
    let msg = raw_message.strip_prefix(&format!("[CQ:at,qq={self_id}]"))?;
    let msg: Vec<&str> = msg.trim().split(' ').collect();
    let reply = |v| Some(op_send_group_msg(group_id, v));
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
            let n = (elapse(0.0) * 864e5) as usize % 20 + 3;
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
        "订阅通知" => {
            db_groups_insert(group_id);
            reply("订阅成功")
        }
        "设置回复" => {
            let (k, v) = (msg[1].into(), msg[2].into());
            REPLIES.lock().await.insert(k, v);
            reply("记住啦")
        }
        k if REPLIES.lock().await.contains_key(k) => reply(&REPLIES.lock().await[k]),
        _ => reply("未知指令"),
    }
}

static BROADCAST: Lazy<Sender<String>> = Lazy::new(|| broadcast::channel(16).0);

async fn ws_handler(ws: WebSocket) {
    let (sender, mut receiver) = ws.split();
    let sender = Arc::new(Mutex::new(sender));
    let mut task1 = tokio::spawn({
        let sender = sender.clone();
        let mut broadcast = BROADCAST.subscribe();
        async move {
            while let Ok(v) = broadcast.recv().await {
                if sender.lock().await.send(Message::Text(v)).await.is_err() {
                    break;
                }
            }
        }
    });
    let mut task2 = tokio::spawn(async move {
        while let Some(Ok(Message::Text(event))) = receiver.next().await {
            if let Some(v) = event_handler(serde_json::from_str(&event).unwrap()).await {
                if sender.lock().await.send(Message::Text(v)).await.is_err() {
                    break;
                }
            }
        }
    });
    tokio::select! {
        _ = (&mut task1) => task2.abort(),
        _ = (&mut task2) => task1.abort(),
    };
}

pub fn service() -> Router {
    db_init();
    Router::new().route(
        "/qqbot",
        MethodRouter::new().get(|u: WebSocketUpgrade| async { u.on_upgrade(ws_handler) }),
    )
}

async fn notify(msg: &str) {
    for group_id in db_groups_get() {
        BROADCAST.send(op_send_group_msg(group_id, msg)).ok();
    }
}

struct UpNotify {
    name: &'static str,
    pkg_id: &'static str,
    last: Mutex<String>,
}

impl UpNotify {
    async fn query(pkg_id: &str) -> String {
        let client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none());
        let client = client.build().unwrap();
        let url = format!("https://community.chocolatey.org/api/v2/package/{pkg_id}");
        let ret = client.get(&url).send().await.unwrap().text().await.unwrap();
        let ret = ret.rsplit_once(".nupkg").unwrap().0;
        let ret = ret.rsplit_once('/').unwrap().1;
        ret.split_once('.').unwrap().1.to_string()
    }

    async fn trigger(&self) {
        let v = Self::query(self.pkg_id).await;
        let changed = {
            let last = self.last.lock().await;
            !last.is_empty() && *last != v
        };
        if changed {
            notify(&format!("{} {v} released!", self.name)).await;
            *self.last.lock().await = v;
        }
    }

    fn new(name: &'static str, pkg_id: &'static str) -> Self {
        Self {
            name,
            pkg_id,
            last: Mutex::new(String::new()),
        }
    }
}

static TICKER: Lazy<Mutex<Ticker>> =
    Lazy::new(|| Mutex::new(Ticker::new_p8(&[(-1, 20, 0), (-1, 50, 0)])));
pub async fn tick() {
    if !TICKER.lock().await.tick() {
        return;
    }

    static UP_CHROME: Lazy<UpNotify> = Lazy::new(|| UpNotify::new("Chrome", "googlechrome"));
    static UP_VSCODE: Lazy<UpNotify> = Lazy::new(|| UpNotify::new("VSCode", "vscode"));
    static UP_RUST: Lazy<UpNotify> = Lazy::new(|| UpNotify::new("Rust", "rust"));
    let _ = tokio::join!(
        tokio::spawn(UP_CHROME.trigger()),
        tokio::spawn(UP_VSCODE.trigger()),
        tokio::spawn(UP_RUST.trigger()),
    );
}
