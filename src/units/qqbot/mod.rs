use crate::db;
use crate::ticker::Ticker;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::routing::MethodRouter;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::Mutex;

trait OptionToResult<T> {
    fn e(self) -> AnyResult<T>;
}
impl<T> OptionToResult<T> for Option<T> {
    fn e(self) -> AnyResult<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(AnyError("Option is None".into())),
        }
    }
}
struct AnyError(String);
impl<T: fmt::Debug> From<T> for AnyError {
    fn from(i: T) -> Self {
        AnyError(format!("{:?}", i))
    }
}
impl fmt::Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
type AnyResult<T> = std::result::Result<T, AnyError>;
macro_rules! touch {
    ($result:expr) => {{
        match $result {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[touched error] {}:{} {}", file!(), line!(), e);
                return Default::default();
            }
        }
    }};
}

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

async fn fetch(url: &str) -> AnyResult<String> {
    Ok(reqwest::get(url).await?.text().await?)
}
async fn fetch_json(url: &str, path: &str) -> AnyResult<String> {
    let text = fetch(url).await?;
    let mut v = &serde_json::from_str::<serde_json::Value>(&text)?;
    for k in path.split('.') {
        v = v.get(k).ok_or("field not found")?;
    }
    Ok(v.to_string())
}

fn elapse(time: f64) -> f64 {
    // javascript: new Date("2001.01.01 06:00").getTime()
    let epoch = SystemTime::UNIX_EPOCH;
    let now = SystemTime::now().duration_since(epoch).unwrap().as_millis() as f64;
    (now - time) / 864e5
}

fn op_send_group_msg(group_id: i64, msg: &str) -> String {
    let msg = "[BOT] ".to_string() + msg;
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

/// generate reply from message parts, returns `""` for inner error (by `touch` macro)
async fn gen_reply(msg: Vec<&str>) -> String {
    match msg[..] {
        ["kk单身多久了"] => format!("kk已连续单身 {:.3} 天了", elapse(10485432e5)),
        ["暑假倒计时"] => format!("距 2022 暑假仅 {:.3} 天", -elapse(16574688e5)),
        ["高考倒计时"] => format!("距 2022 高考仅 {:.3} 天", -elapse(16545636e5)),
        ["驶向深蓝"] => {
            let url = "https://api.lovelive.tools/api/SweetNothings?genderType=M";
            touch!(fetch(url).await)
        }
        ["吟诗"] => {
            let url = "https://v1.jinrishici.com/all.json";
            touch!(fetch_json(url, "content").await)
        }
        ["新闻"] => {
            let n = (elapse(0.0) * 864e5) as usize % 20 + 3;
            let r = touch!(fetch("https://m.cnbeta.com/wap").await);
            let r = r.split("htm\">").nth(n).and_then(|v| v.split_once('<'));
            touch!(r.ok_or("process text failed")).0.into()
        }
        ["比特币"] | ["BTC"] => {
            let url = "https://chain.so/api/v2/get_info/BTC";
            let price = touch!(fetch_json(url, "data.price").await);
            format!("比特币当前价格 {} 美元", price.trim_matches('0'))
        }
        ["垃圾分类", i] => {
            let url = format!("https://api.muxiaoguo.cn/api/lajifl?m={i}");
            match fetch_json(&url, "data.type").await {
                Ok(v) => format!("{i} {v}"),
                Err(_) => format!("鬼知道 {i} 是什么垃圾呢"),
            }
        }
        ["聊天", i, ..] => {
            let url = format!("https://api.ownthink.com/bot?spoken={i}");
            touch!(fetch_json(&url, "data.info.text").await)
        }
        ["订阅通知", v] => {
            db_groups_insert(touch!(v.parse()));
            format!("已为当前群 {v} 订阅通知")
        }
        ["设置回复", k, v] => {
            let (k, v) = (k.into(), v.into()); // pregenerate pair to avoid mutex posion error
            REPLIES.lock().await.insert(k, v);
            "记住啦".into()
        }
        [k, ..] => match REPLIES.lock().await.get(k) {
            Some(v) => v.clone(),
            None => "指令有误".into(),
        },
        [] => "你没有附加任何指令呢".into(),
    }
}

/// `notify()` -> `BROADCAST` -> `task_broadcast` in `ws_handler()`
static BROADCAST: Lazy<Sender<String>> = Lazy::new(|| broadcast::channel(16).0);

async fn ws_handler(ws: WebSocket) {
    let (sender, mut receiver) = ws.split();
    let sender = Arc::new(Mutex::new(sender));

    /// returns (group_id, message_parts) or `None` if don't care (chaos events)
    fn parse_event(event: &serde_json::Value) -> Option<(i64, Vec<&str>)> {
        let self_id = event.get("self_id")?.as_i64()?;
        let group_id = event.get("group_id")?.as_i64()?;
        let raw_message = event.get("raw_message")?.as_str()?;
        let msg = raw_message.strip_prefix(&format!("[CQ:at,qq={self_id}]"))?;
        Some((group_id, msg.trim().split(' ').collect()))
    }

    let mut task_broadcast = tokio::spawn({
        let sender = sender.clone();
        async move {
            let mut broadcast = BROADCAST.subscribe();
            loop {
                let v = touch!(broadcast.recv().await);
                touch!(sender.lock().await.send(Message::Text(v)).await);
            }
        }
    });

    let mut task_reply = tokio::spawn(async move {
        while let Some(Ok(Message::Text(event))) = receiver.next().await {
            let event = touch!(serde_json::from_str(&event));
            let (group_id, msg) = match parse_event(&event) {
                Some(v) => v,
                None => continue,
            };
            let v = gen_reply(msg).await;
            let v = op_send_group_msg(group_id, if v.is_empty() { "内部错误" } else { &v });
            touch!(sender.lock().await.send(Message::Text(v)).await);
        }
    });

    // if any one of the tasks exit, abort another
    tokio::select! {
        _ = (&mut task_reply) => task_broadcast.abort(),
        _ = (&mut task_broadcast) => task_reply.abort(),
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
        touch!(BROADCAST.send(op_send_group_msg(group_id, msg)));
    }
}

struct UpNotify {
    name: &'static str,
    pkg_id: &'static str,
    last: Mutex<String>,
}

impl UpNotify {
    async fn query(pkg_id: &str) -> AnyResult<String> {
        let client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none());
        let client = client.build().unwrap();
        let url = format!("https://community.chocolatey.org/api/v2/package/{pkg_id}");
        let ret = client.get(&url).send().await?.text().await?;
        let ret = ret.rsplit_once(".nupkg").e()?.0;
        let ret = ret.rsplit_once('/').e()?.1;
        Ok(ret.split_once('.').e()?.1.to_string())
    }

    async fn trigger(&self) {
        let v = touch!(Self::query(self.pkg_id).await);
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

    let _ = tokio::join!(UP_CHROME.trigger(), UP_VSCODE.trigger(), UP_RUST.trigger());
}
