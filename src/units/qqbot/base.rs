use super::gen_reply;
use crate::{care, db, utils::slot};
use anyhow::Result;
use axum::extract::Form;
use axum::response::{Html, Redirect};
use once_cell::sync::Lazy;
use ricq::device::Device;
use ricq::ext::reconnect::{Connector, DefaultConnector};
use ricq::handler::QEvent;
use ricq::msg::elem::RQElem;
use ricq::msg::MessageChain;
use ricq::version::{get_version, Protocol};
use ricq::{Client, LoginResponse, QRCodeImageFetch, QRCodeState};
use serde::Deserialize;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::mpsc::Receiver;
use tokio::time::Duration;

macro_rules! push_log {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        let mut log = LOG.lock().unwrap();
        let max_len = 256;
        if log.len() > max_len{
            log.pop_front();
        }
        let epoch = SystemTime::UNIX_EPOCH;
        let now = SystemTime::now().duration_since(epoch).unwrap().as_millis();
        log.push_back(format!("[{now}] {s}"));
    }}
}

const K_TOKEN: &str = "token_json";
const K_DEVICE: &str = "device_json";

fn db_init() {
    db!("CREATE TABLE qqbot_cfg (k TEXT UNIQUE, v BLOB)").ok();
    db!("CREATE TABLE qqbot_groups (group_id INTEGER UNIQUE)").ok();
}
fn db_cfg_set(k: &str, v: &[u8]) {
    db!("INSERT OR REPLACE INTO qqbot_cfg VALUES (?1, ?2)", [k, v]).unwrap();
}
fn db_cfg_get(k: &str) -> Option<Vec<u8>> {
    let result = db!("SELECT v FROM qqbot_cfg WHERE k = ?", [k], (0));
    result.unwrap().pop()?.0
}
fn db_cfg_get_text(k: &str) -> Option<String> {
    Some(String::from_utf8(db_cfg_get(k)?).unwrap())
}
fn db_groups_get() -> Vec<i64> {
    let result = db!("SELECT * FROM qqbot_groups", [], (0));
    result.unwrap().into_iter().map(|r| r.0).collect()
}
pub fn db_groups_insert(group_id: i64) {
    db!("INSERT INTO qqbot_groups VALUES (?)", [group_id]).unwrap();
}

#[derive(Deserialize)]
pub struct Submit {
    key: String,
    value: String,
}

pub async fn post_handler(form: Form<Submit>) -> Redirect {
    db_cfg_set(&form.key, form.value.as_bytes());
    Redirect::to("/qqbot")
}

pub async fn get_handler() -> Html<String> {
    const PAGE: [&str; 2] = slot(include_str!("page.html"));
    let mut body = PAGE[0].to_string();
    #[allow(clippy::significant_drop_in_scrutinee)]
    for line in LOG.lock().unwrap().iter() {
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

static LOG: Lazy<Mutex<VecDeque<String>>> = Lazy::new(Default::default);
static QR: Lazy<Mutex<Vec<u8>>> = Lazy::new(Default::default);
static CLIENT: Lazy<Arc<ricq::Client>> = Lazy::new(|| {
    push_log!("init client");
    db_init();
    let device = match db_cfg_get_text(K_DEVICE) {
        Some(v) => serde_json::from_str(&v).unwrap(),
        None => {
            let device = Device::random();
            db_cfg_set(K_DEVICE, serde_json::to_string(&device).unwrap().as_bytes());
            device
        }
    };
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let client = Arc::new(Client::new(device, get_version(Protocol::AndroidWatch), tx));
    tokio::spawn(async {
        tokio::join!(
            async {
                let stream = DefaultConnector.connect(&CLIENT).await.unwrap();
                CLIENT.start(stream).await;
            },
            launch(rx)
        )
    });
    client
});

async fn launch(mut rx: Receiver<QEvent>) -> Result<()> {
    // waiting for connected
    while CLIENT.get_status() == 0 {
        tokio::time::sleep(Duration::from_secs_f32(0.2)).await;
    }
    tokio::task::yield_now().await;
    push_log!("server connected");

    // # Tips about Login
    // 1. Run on local host, login by qrcode.
    // 2. Run on remote, copy device_json and token_json to database.
    // 3. Restart remote server.
    if let Some(v) = db_cfg_get_text(K_TOKEN) {
        let token = serde_json::from_str(&v)?;
        CLIENT.token_login(token).await?;
        push_log!("login with token succeed");
    } else {
        let mut qr_resp = CLIENT.fetch_qrcode().await?;
        let mut img_sig = Vec::new();
        loop {
            async fn load_qr(fetching: QRCodeImageFetch, img_sig: &mut Vec<u8>) {
                push_log!("qrcode fetched");
                *QR.lock().unwrap() = fetching.image_data.to_vec();
                *img_sig = fetching.sig.to_vec();
            }
            match qr_resp {
                QRCodeState::ImageFetch(inner) => load_qr(inner, &mut img_sig).await,
                QRCodeState::Timeout => {
                    push_log!("qrcode timeout");
                    if let QRCodeState::ImageFetch(inner) = CLIENT.fetch_qrcode().await? {
                        load_qr(inner, &mut img_sig).await;
                    }
                }
                QRCodeState::Confirmed(inner) => {
                    push_log!("qrcode confirmed");
                    let login_resp = CLIENT
                        .qrcode_login(&inner.tmp_pwd, &inner.tmp_no_pic_sig, &inner.tgt_qr)
                        .await?;
                    if let LoginResponse::DeviceLockLogin { .. } = login_resp {
                        CLIENT.device_lock_login().await?;
                    }
                    push_log!("login with qrcode succeed");
                    let token = serde_json::to_string(&CLIENT.gen_token().await)?;
                    db_cfg_set(K_TOKEN, token.as_bytes());
                    break;
                }
                QRCodeState::WaitingForScan => push_log!("qrcode waiting for scan"),
                QRCodeState::WaitingForConfirm => push_log!("qrcode waiting for confirm"),
                QRCodeState::Canceled => push_log!("qrcode canceled"),
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
            qr_resp = CLIENT.query_qrcode_result(&img_sig).await?;
        }
    }
    // instead of `ricq::ext::common::after_login`
    CLIENT.register_client().await?;
    CLIENT.refresh_status().await?;

    QR.lock().unwrap().clear();

    tokio::join!(
        async {
            loop {
                CLIENT.do_heartbeat().await;
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        },
        async {
            loop {
                care!(on_event(rx.recv().await.unwrap()).await).ok();
            }
        }
    );
    Ok(())
}

fn text_msg(content: String) -> MessageChain {
    MessageChain::new(ricq::msg::elem::Text::new(format!("[BOT] {content}")))
}

async fn on_event(event: QEvent) -> Result<()> {
    match event {
        QEvent::GroupMessage(e) => {
            match { e.inner.elements.0.get(0) }.map(|v| RQElem::from((v).clone())) {
                Some(RQElem::At(v)) if v.target == CLIENT.uin().await => {}
                _ => return Ok(()), // it's not my business!
            }
            let msg = e.inner.elements.to_string();
            let msg: Vec<&str> = msg.split_whitespace().skip(1).collect();
            let reply = gen_reply(msg).await?;
            CLIENT
                .send_group_message(e.inner.group_code, text_msg(reply))
                .await?;
        }
        QEvent::GroupMessageRecall(_) => {}
        QEvent::Login(e) => push_log!("login {e}"),
        _ => {}
    }
    Ok(())
}

pub async fn notify(msg: String) -> Result<()> {
    let msg_chain = text_msg(msg);
    for group in db_groups_get() {
        CLIENT.send_group_message(group, msg_chain.clone()).await?;
    }
    Ok(())
}
