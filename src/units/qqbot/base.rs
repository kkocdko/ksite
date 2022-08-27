//! Provide login, token storage and other base functions.
use super::gen_reply;
use crate::utils::read_body;
use crate::{care, db, include_page};
use anyhow::Result;
use axum::extract::{RawBody, RawQuery};
use axum::response::Html;
use once_cell::sync::Lazy;
use ricq::client::{Connector as _, DefaultConnector, NetworkStatus};
use ricq::handler::QEvent;
use ricq::msg::elem::RQElem;
use ricq::msg::MessageChain;
use ricq::structs::GroupMessage;
use ricq::{Client, Device, LoginResponse, Protocol, QRCodeState};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

macro_rules! push_log {
    ($fmt:literal $(, $($arg:tt)+)?) => {{
        let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
        push_log_(format!(concat!("{} | ", $fmt), now, $($($arg)+)?));
    }}
}
fn push_log_(v: String) {
    let mut log = LOG.lock().unwrap();
    if log.len() >= 128 {
        log.drain(..32);
    }
    log.push(v);
}

const K_DEVICE: &str = "device_json";
const K_TOKEN: &str = "token_json";

fn db_init() {
    db!("CREATE TABLE qqbot_cfg (k TEXT PRIMARY KEY, v BLOB)").ok();
    db!("CREATE TABLE qqbot_groups (group_id INTEGER PRIMARY KEY)").ok();
}
fn db_cfg_set(k: &str, v: Vec<u8>) {
    db!("REPLACE INTO qqbot_cfg VALUES (?1, ?2)", [k, v]).unwrap();
}
fn db_cfg_get(k: &str) -> Option<Vec<u8>> {
    db!("SELECT v FROM qqbot_cfg WHERE k = ?", [k], ^|r| r.get(0)).ok()
}
fn db_cfg_get_text(k: &str) -> Option<String> {
    Some(String::from_utf8(db_cfg_get(k)?).unwrap())
}
fn db_groups_get() -> Vec<i64> {
    db!("SELECT * FROM qqbot_groups", [], |r| r.get(0)).unwrap()
}
pub fn db_groups_insert(group_id: i64) {
    db!("REPLACE INTO qqbot_groups VALUES (?)", [group_id]).unwrap();
}
pub fn _db_groups_delete(group_id: i64) -> bool {
    db!("DELETE FROM qqbot_groups WHERE group_id = ?", [group_id]).is_ok()
}

pub async fn post_handler(q: RawQuery, RawBody(body): RawBody) {
    let q = q.0.unwrap();
    let k = q.split_once('=').unwrap().1;
    let v = read_body(body).await;
    db_cfg_set(k, v);
}

pub async fn get_handler() -> Html<String> {
    const PAGE: [&str; 2] = include_page!("page.html");
    let mut body = PAGE[0].to_string();
    for line in LOG.lock().unwrap().iter().rev() {
        body += line;
        body += "\n";
    }
    body.push_str(PAGE[1]);
    Html(body)
}

pub fn get_login_qr() -> Vec<u8> {
    CLIENT.get_status(); // init client
    QR.lock().unwrap().clone()
}

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static QR: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static CLIENT: Lazy<Arc<Client>> = Lazy::new(|| {
    push_log!("init client");
    db_init();
    let device = match db_cfg_get_text(K_DEVICE) {
        Some(v) => serde_json::from_str(&v).unwrap(),
        None => {
            let device = Device::random();
            db_cfg_set(
                K_DEVICE,
                serde_json::to_string(&device).unwrap().into_bytes(),
            );
            device
        }
    };
    let client = Arc::new(Client::new(device, Protocol::IPad.into(), MyHandler));
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let mut last = UNIX_EPOCH.elapsed().unwrap().as_secs();
        loop {
            tokio::select! {
                _ = async {
                    push_log!("try to connect");
                    let stream = DefaultConnector.connect(&CLIENT).await?;
                    CLIENT.start(stream).await;
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
            CLIENT.stop(NetworkStatus::NetworkOffline);
            let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
            if now - last < 60 {
                push_log!("reconnection was stopped, overfrequency");
                return;
            }
            last = now;
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
    client
});

async fn launch() -> Result<()> {
    // waiting for connected
    while CLIENT.get_status() == 0 {
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    push_log!("server connected");

    // # Tips about Login
    // 1. Run on local host, login by qrcode.
    // 2. Run on remote, copy device_json and token_json to database.
    // 3. Restart remote server.
    if let Some(v) = db_cfg_get_text(K_TOKEN) {
        let token = serde_json::from_str(&v)?;
        CLIENT.token_login(token).await?;
        push_log!("login by token succeeded");
    } else {
        let mut qr_resp = CLIENT.fetch_qrcode().await?;
        let mut img_sig = Vec::new();
        loop {
            match qr_resp {
                QRCodeState::ImageFetch(inner) => {
                    push_log!("qrcode fetched");
                    *QR.lock().unwrap() = inner.image_data.to_vec();
                    img_sig = inner.sig.to_vec();
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
                    push_log!("login by qrcode succeeded");
                    let token = serde_json::to_string(&CLIENT.gen_token().await)?;
                    db_cfg_set(K_TOKEN, token.into_bytes());
                    break;
                }
                QRCodeState::WaitingForScan => push_log!("qrcode waiting for scan"),
                QRCodeState::WaitingForConfirm => push_log!("qrcode waiting for confirm"),
                QRCodeState::Canceled => push_log!("qrcode canceled"),
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
            qr_resp = CLIENT.query_qrcode_result(&img_sig).await?;
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    // instead of `ricq::ext::common::after_login`
    CLIENT.register_client().await?;
    CLIENT.refresh_status().await?;

    QR.lock().unwrap().clear();
    Ok(())
}

fn text_msg(content: String) -> MessageChain {
    MessageChain::new(ricq::msg::elem::Text::new(format!("[BOT] {content}")))
}

async fn on_event(event: QEvent) -> Result<()> {
    static RECENT: Mutex<Vec<GroupMessage>> = Mutex::new(Vec::new());
    match event {
        QEvent::GroupMessage(e) => {
            if matches!(
                e.inner.elements.0.get(0).map(|v| RQElem::from(v.clone())),
                Some(RQElem::At(v)) if v.target == CLIENT.uin().await
            ) {
                let msg = e.inner.elements.to_string();
                let msg_parts = msg.split_whitespace().skip(1).collect();
                let reply = care!(gen_reply(msg_parts).await)?;
                CLIENT
                    .send_group_message(e.inner.group_code, text_msg(reply))
                    .await?;
            }
            let time = e.inner.time;
            let mut recent = RECENT.lock().unwrap();
            recent.push(e.inner);
            let len = recent.len();
            // filter out the expired messages if these conditions are all true:
            // 1. message storage size is reached the limit
            // 2. size % 8 == 0, throttling while extreme scene
            if len >= 64 && len % 8 == 0 {
                // messages sent 2 minutes ago cannot be recalled
                recent.retain(|v| time - v.time <= 120);
                // push_log!("cleaned {} expired messages", len - recent.len());
            }
        }
        QEvent::GroupMessageRecall(e) => {
            let recent = RECENT.lock().unwrap();
            if let Some(v) = recent.iter().find(|v| v.seqs.contains(&e.inner.msg_seq)) {
                push_log!(
                    r#"recalled message = {{ group: "{}", user: "{}", content: "{}" }}"#,
                    v.group_name,
                    v.group_card,
                    v.elements.to_string()
                );
            }
        }
        QEvent::Login(e) => push_log!("login {}", e),
        _ => {}
    }
    Ok(())
}
struct MyHandler;
impl ricq::handler::Handler for MyHandler {
    fn handle<'a: 'b, 'b>(&'a self, e: QEvent) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>> {
        Box::pin(async {
            // add timeout here?
            care!(on_event(e).await).ok();
        })
    }
}

pub async fn notify(msg: String) -> Result<()> {
    let msg_chain = text_msg(msg);
    for group in db_groups_get() {
        CLIENT.send_group_message(group, msg_chain.clone()).await?;
    }
    Ok(())
}
