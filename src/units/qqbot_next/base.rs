use super::gen_reply;
use crate::care;
use crate::db;
use crate::utils::OptionResult;
use anyhow::Result;
use once_cell::sync::Lazy;
use ricq::client::Client;
use ricq::device::Device;
use ricq::ext::common::after_login;
use ricq::handler::QEvent;
use ricq::msg::elem::RQElem;
use ricq::msg::MessageChain;
use ricq::version::{get_version, Protocol};
use ricq::{LoginResponse, QRCodeImageFetch, QRCodeState};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Duration;

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

pub async fn fetch_text(url: &str) -> Result<String> {
    Ok(reqwest::get(url).await?.text().await?)
}

pub async fn fetch_json(url: &str, pointer: &str) -> Result<String> {
    let text = fetch_text(url).await?;
    let v = serde_json::from_str::<serde_json::Value>(&text)?;
    let v = v.pointer(pointer).e()?.to_string();
    Ok(v.trim_matches('"').to_string())
}

pub fn elapse(time: f64) -> f64 {
    // javascript: new Date("2001.01.01 06:00").getTime()
    let epoch = SystemTime::UNIX_EPOCH;
    let now = SystemTime::now().duration_since(epoch).unwrap().as_millis() as f64;
    (now - time) / 864e5 // unit: days
}

static QR: Lazy<Mutex<Vec<u8>>> = Lazy::new(|| Mutex::new(Vec::new()));
static CLIENT: Lazy<Arc<ricq::Client>> = Lazy::new(|| {
    db_init();
    let device = {
        let db_key = "device_json";
        match db_cfg_get_text(db_key) {
            Some(v) => serde_json::from_str(&v).unwrap(),
            None => {
                let device = Device::random();
                db_cfg_set(db_key, serde_json::to_string(&device).unwrap().as_bytes());
                device
            }
        }
    };
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let client = Arc::new(Client::new(device, get_version(Protocol::AndroidWatch), tx));
    tokio::spawn(async move {
        loop {
            on_event(rx.recv().await.unwrap()).await;
        }
    });
    tokio::spawn(async move {
        let stream = TcpStream::connect(CLIENT.get_address()).await.unwrap();
        CLIENT.start(stream).await
    });
    client
});

pub async fn get_qr() -> Vec<u8> {
    QR.lock().await.clone()
}

pub async fn launch() -> Result<()> {
    // waiting for connected
    while CLIENT.get_status() == 0 {
        tokio::time::sleep(Duration::from_secs_f32(0.1)).await;
    }
    tokio::task::yield_now().await;
    push_log("server connected");

    let mut qr_resp = CLIENT.fetch_qrcode().await?;
    let mut img_sig = Vec::new();
    loop {
        async fn load_qr(fetching: QRCodeImageFetch, img_sig: &mut Vec<u8>) {
            push_log("qrcode fetched");
            *QR.lock().await = fetching.image_data.to_vec();
            *img_sig = fetching.sig.to_vec();
        }
        match qr_resp {
            QRCodeState::ImageFetch(inner) => load_qr(inner, &mut img_sig).await,
            QRCodeState::Timeout => {
                push_log("qrcode timeout");
                if let QRCodeState::ImageFetch(inner) = CLIENT.fetch_qrcode().await? {
                    load_qr(inner, &mut img_sig).await;
                }
            }
            QRCodeState::Confirmed(inner) => {
                push_log("qrcode confirmed");
                QR.lock().await.clear();
                let login_resp = CLIENT
                    .qrcode_login(&inner.tmp_pwd, &inner.tmp_no_pic_sig, &inner.tgt_qr)
                    .await?;
                if let LoginResponse::DeviceLockLogin { .. } = login_resp {
                    CLIENT.device_lock_login().await?;
                }
                push_log("login succeed");
                break;
            }
            QRCodeState::WaitingForScan => push_log("qrcode waiting for scan"),
            QRCodeState::WaitingForConfirm => push_log("qrcode waiting for confirm"),
            QRCodeState::Canceled => push_log("qrcode canceled"),
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
        qr_resp = CLIENT.query_qrcode_result(&img_sig).await?;
    }
    after_login(&CLIENT).await;
    loop {
        CLIENT.heartbeat().await.unwrap();
    }
}

async fn on_event(event: QEvent) {
    match event {
        QEvent::GroupMessage(e) => {
            match { e.inner.elements.0.get(0) }.map(|v| RQElem::from((v).clone())) {
                Some(RQElem::At(v)) if v.target == e.client.uin().await => {}
                _ => return, // it's not my business!
            }
            let msg = e.inner.elements.to_string();
            let msg: Vec<&str> = msg.split_whitespace().skip(1).collect();
            let reply = care!(gen_reply(msg).await, return);
            CLIENT
                .send_group_message(e.inner.group_code, text_msg(reply))
                .await
                .unwrap();
        }
        QEvent::Login(e) => push_log(&format!("login {e}")),
        _ => {}
    }
}

fn text_msg(content: String) -> MessageChain {
    MessageChain::new(ricq::msg::elem::Text::new(format!("[BOT] {content}")))
}

pub async fn notify(msg: String) {
    let msg_chain = text_msg(msg);
    for group in db_groups_get() {
        CLIENT
            .send_group_message(group, msg_chain.clone())
            .await
            .unwrap();
    }
}

fn push_log(_t: &str) {}
