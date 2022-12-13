//! QQ robot for fun.

mod command;
use crate::auth::auth_layer;
use crate::ticker::Ticker;
use crate::utils::{fetch_text, log_escape, OptionResult};
use crate::{care, include_page};
use anyhow::Result;
use axum::body::Bytes;
use axum::extract::RawQuery;
use axum::middleware;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use ricq::client::{Connector as _, DefaultConnector, NetworkStatus};
use ricq::handler::QEvent;
use ricq::msg::MessageChain;
use ricq::{Client, Device, LoginResponse, Protocol, QRCodeState};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

mod db {
    use crate::db;

    pub const K_DEVICE: &str = "device_json";
    pub const K_TOKEN: &str = "token_json";
    pub const K_NOTIFY_GROUPS: &str = "notify_groups";

    pub fn init() {
        db!("
            CREATE TABLE IF NOT EXISTS qqbot_log
            (time INTEGER, content BLOB);
            CREATE TABLE IF NOT EXISTS qqbot_cfg
            (k BLOB PRIMARY KEY, v BLOB);
            INSERT OR IGNORE INTO qqbot_cfg VALUES
            (cast('notify_groups' as BLOB), X'');
        ")
        .unwrap();
        // format: notify_groups = b"7652318,17931963,123132"
    }

    pub fn log_insert(content: &str) {
        db!(
            "
            INSERT INTO qqbot_log
            VALUES (strftime('%s', 'now'), ?1)
            ",
            [content.as_bytes()]
        )
        .unwrap();
    }

    pub fn log_get() -> Vec<(u64, String)> {
        db!(
            "
            SELECT * FROM qqbot_log
            ",
            [],
            |r| Ok((r.get(0)?, String::from_utf8(r.get(1)?).unwrap()))
        )
        .unwrap()
    }

    pub fn log_clean() {
        db!("
            DELETE FROM qqbot_log
            WHERE strftime('%s', 'now') - time > 3600 * 24 * 3
        ")
        .unwrap();
    }

    pub fn cfg_set(k: &str, v: &[u8]) {
        db!(
            "
            REPLACE INTO qqbot_cfg
            VALUES (?1, ?2)
            ",
            [k.as_bytes(), v]
        )
        .unwrap();
    }

    pub fn cfg_get(k: &str) -> Option<Vec<u8>> {
        db!(
            "
            SELECT v FROM qqbot_cfg
            WHERE k = ?
            ",
            [k.as_bytes()],
            &|r| r.get(0)
        )
        .ok()
    }

    pub fn cfg_get_str(k: &str) -> Option<String> {
        Some(String::from_utf8(cfg_get(k)?).unwrap())
    }
}

fn push_log_(v: &str) {
    db::log_insert(&log_escape(v));
}
macro_rules! push_log {
    ($($arg:tt)*) => {{
        push_log_(&format!($($arg)*));
    }};
}

static QR: Mutex<Bytes> = Mutex::new(Bytes::new());
static CLIENT: Lazy<Arc<Client>> = Lazy::new(|| {
    push_log!("init client");
    let device = match db::cfg_get_str(db::K_DEVICE) {
        Some(v) => serde_json::from_str(&v).unwrap(),
        None => {
            let device = Device::random();
            db::cfg_set(
                db::K_DEVICE,
                serde_json::to_string(&device).unwrap().as_bytes(),
            );
            device
        }
    };
    let client = Arc::new(Client::new(
        device,
        Protocol::MacOS.into(),
        on_event as fn(_) -> _,
    ));
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let mut last = UNIX_EPOCH.elapsed().unwrap().as_secs();
        loop {
            tokio::select! {
                _ = async {
                    CLIENT.start(DefaultConnector.connect(&CLIENT).await?).await;
                    push_log!("offline, fn start returned");
                    anyhow::Ok(())
                } => {}
                _ = async {
                    launch().await?;
                    CLIENT.do_heartbeat().await;
                    push_log!("offline, fn do_heartbeat returned");
                    anyhow::Ok(())
                } => {}
            };
            CLIENT.stop(NetworkStatus::Unknown);
            let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
            if now - last < 30 {
                push_log!("reconnection was stopped, overfrequency");
                return;
            }
            last = now;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    client
});

async fn launch() -> Result<()> {
    // waiting for connected
    while CLIENT.get_status() != NetworkStatus::Unknown as u8 {
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    push_log!("server connected");

    // # Tips about Login
    // 1. Run on local host, login by qrcode.
    // 2. Run on remote, copy device_json and token_json to database.
    // 3. Restart remote server.
    if let Some(v) = db::cfg_get_str(db::K_TOKEN) {
        let token = serde_json::from_str(&v)?;
        CLIENT.token_login(token).await?;
        push_log!("login by token");
    } else {
        let mut qr_resp = CLIENT.fetch_qrcode().await?;
        let mut img_sig = Bytes::new();
        loop {
            match qr_resp {
                QRCodeState::ImageFetch(inner) => {
                    push_log!("qrcode fetched");
                    *QR.lock().unwrap() = inner.image_data;
                    img_sig = inner.sig;
                }
                QRCodeState::Timeout => {
                    push_log!("qrcode timeout");
                    qr_resp = CLIENT.fetch_qrcode().await?;
                    continue;
                }
                QRCodeState::Confirmed(inner) => {
                    push_log!("qrcode confirmed");
                    let login_resp = CLIENT
                        .qrcode_login(&inner.tmp_pwd, &inner.tmp_no_pic_sig, &inner.tgt_qr)
                        .await?;
                    if let LoginResponse::DeviceLockLogin { .. } = login_resp {
                        CLIENT.device_lock_login().await?;
                    }
                    push_log!("login by qrcode");
                    break;
                }
                QRCodeState::WaitingForScan => push_log!("qrcode waiting for scan"),
                QRCodeState::WaitingForConfirm => push_log!("qrcode waiting for confirm"),
                QRCodeState::Canceled => push_log!("qrcode canceled"),
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
            qr_resp = CLIENT.query_qrcode_result(&img_sig).await?;
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }
    // instead of `ricq::ext::common::after_login`
    CLIENT.register_client().await?;
    CLIENT.refresh_status().await?;

    // clear qr code (the `clear()` will not release capacity)
    std::mem::take(&mut *QR.lock().unwrap());

    // save new token
    tokio::time::sleep(Duration::from_secs(1)).await;
    let token = CLIENT.gen_token().await;
    db::cfg_set(db::K_TOKEN, serde_json::to_string(&token)?.as_bytes());

    Ok(())
}

fn bot_msg(content: &str) -> MessageChain {
    MessageChain::new(ricq::msg::elem::Text::new(format!("[BOT] {content}")))
}

async fn prefix_matched(i: &str) -> bool {
    static PREFIX: Mutex<String> = Mutex::new(String::new());
    let is_prefix_uninit = { PREFIX.lock().unwrap().is_empty() };
    let prefix = if is_prefix_uninit {
        let v = format!("[@{}]", CLIENT.account_info.read().await.nickname);
        let mut prefix = PREFIX.lock().unwrap();
        *prefix = v;
        prefix
    } else {
        PREFIX.lock().unwrap()
    };
    i.starts_with(&*prefix)
}

async fn on_event(event: QEvent) {
    #[allow(clippy::type_complexity)]
    static RECENT: Mutex<Vec<(i32, Vec<i32>, String, String, String)>> = Mutex::new(Vec::new());
    match event {
        QEvent::GroupMessage(e) => {
            let e = e.inner;
            let msg = e.elements.to_string();
            if prefix_matched(&msg).await {
                let msg_parts = msg.split_whitespace().skip(1).collect();
                if let Ok(reply) =
                    care!(command::on_group_msg(e.group_code, msg_parts, &CLIENT).await)
                {
                    let reply = bot_msg(&reply);
                    let result = CLIENT.send_group_message(e.group_code, reply).await;
                    care!(result).ok();
                }
            }
            // println!("\n\x1b[93m[ksite]\x1b[0m {}", e.inner.elements);
            let mut recent = RECENT.lock().unwrap();
            recent.push((e.time, e.seqs, e.group_name, e.group_card, msg));
            let len = recent.len();
            // size % 8 == 0, throttling while extreme scene
            if len >= 64 && len % 8 == 0 {
                // can't be recalled after 2 minutes
                recent.retain(|v| e.time - v.0 <= 120 + 5);
                // push_log!("cleaned {} expired messages", len - recent.len());
            }
        }
        // the AndroidWatch protocol will not receive recall event
        QEvent::GroupMessageRecall(e) => {
            let recent = RECENT.lock().unwrap();
            if let Some((_, _, group, user, content)) =
                recent.iter().find(|v| v.1.contains(&e.inner.msg_seq))
            {
                push_log!("recalled {group}>{user} {content}");
            }
        }
        _ => {}
    }
}

pub async fn notify(msg: String) -> Result<()> {
    let msg_chain = bot_msg(&msg);
    for part in db::cfg_get_str(db::K_NOTIFY_GROUPS).unwrap().split(',') {
        if let Ok(group) = part.parse() {
            CLIENT.send_group_message(group, msg_chain.clone()).await?;
        }
    }
    Ok(())
}

pub fn service() -> Router {
    db::init();
    CLIENT.get_status(); // init client

    async fn post_handler(q: RawQuery, body: Bytes) {
        let q = q.0.unwrap();
        let k = q.as_str();
        db::cfg_set(k, &body);
    }

    async fn get_handler() -> Html<String> {
        const PAGE: [&str; 2] = include_page!("page.html");
        let mut body = String::new();
        body += PAGE[0];
        for (time, content) in db::log_get().into_iter().rev() {
            writeln!(&mut body, "{time} | {content}").unwrap();
        }
        body += PAGE[1];
        Html(body)
    }

    Router::new()
        .route(
            "/qqbot",
            MethodRouter::new().post(post_handler).get(get_handler),
        )
        .route(
            "/qqbot/qr",
            MethodRouter::new().get(|| async { QR.lock().unwrap().clone() }),
        )
        .layer(middleware::from_fn(auth_layer))
}

struct UpNotify {
    query_url: &'static str,
    last: Mutex<String>,
}

impl UpNotify {
    // https://github.com/rust-lang/rust-clippy/issues/6446
    #[allow(clippy::await_holding_lock)]
    async fn trigger(&self) {
        let v = care!(fetch_text(self.query_url).await, return);
        let v = v
            .rsplit_once(".nupkg")
            .and_then(|v| v.0.rsplit_once('/'))
            .and_then(|v| v.1.split_once('.'));
        let v = care!(v.e(), return).1;
        let mut last = self.last.lock().unwrap();
        if *last == v {
        } else if last.is_empty() {
            *last = v.to_string();
        } else {
            *last = v.to_string();
            drop(last);
            care!(notify(v.to_lowercase()).await, ());
        }
    }
}

macro_rules! up_notify {
    ($pkg_id:literal) => {
        UpNotify {
            query_url: concat!("https://community.chocolatey.org/api/v2/package/", $pkg_id),
            last: Mutex::new(String::new()),
        }
    };
}

static TICKER: Lazy<Ticker> = Lazy::new(|| Ticker::new_p8(&[(-1, 8, 0), (-1, 38, 0)]));
pub async fn tick() {
    if !TICKER.tick() {
        return;
    }

    db::log_clean();

    static UP_CHROME: UpNotify = up_notify!("googlechrome");
    static UP_VSCODE: UpNotify = up_notify!("vscode");
    static UP_RUST: UpNotify = up_notify!("rust");
    let _ = tokio::join!(
        // needless to spawn
        UP_CHROME.trigger(),
        UP_VSCODE.trigger(),
        UP_RUST.trigger()
    );
}
