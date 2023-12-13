//! QQ robot for fun.

mod commands;
use crate::auth::auth_layer;
use crate::utils::{fetch_text, log_escape, str2req, OptionResult};
use crate::{care, include_src, log, ticker};
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
use std::cell::Cell;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

mod db {
    use crate::database::DB;
    use crate::strip_str;
    pub const K_DEVICE: &str = "device_json";
    pub const K_TOKEN: &str = "token_json";
    pub const K_NOTIFY_GROUPS: &str = "notify_groups";
    pub fn init() {
        let db = DB.lock().unwrap();
        let sqls = [
            "CREATE TABLE IF NOT EXISTS qqbot_log (time INTEGER, content BLOB)",
            "CREATE TABLE IF NOT EXISTS qqbot_cfg (k BLOB PRIMARY KEY, v BLOB)",
            "INSERT OR IGNORE INTO qqbot_cfg VALUES (cast('notify_groups' AS BLOB), X'')",
        ];
        for sql in sqls {
            let mut stmd = db.prepare(sql).unwrap();
            stmd.execute(()).unwrap();
        }
        // format: notify_groups = b"7652318,17931963,123132"
    }
    pub fn log_insert(content: &str) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            INSERT INTO qqbot_log VALUES (strftime('%s', 'now'), ?)
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((content.as_bytes(),)).unwrap();
    }
    pub fn log_list() -> Vec<(u64, String)> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT * FROM qqbot_log
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.query_map((), |r| {
            Ok((r.get(0)?, String::from_utf8(r.get(1)?).unwrap()))
        })
        .unwrap()
        .map(|v| v.unwrap())
        .collect()
    }
    pub fn log_clean() {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            DELETE FROM qqbot_log WHERE strftime('%s', 'now') - time > 3600 * 24 * 3
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute(()).unwrap();
    }
    pub fn cfg_set(k: &str, v: &[u8]) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            REPLACE INTO qqbot_cfg VALUES (?, ?)
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((k.as_bytes(), v)).unwrap();
    }
    pub fn cfg_get(k: &str) -> Option<Vec<u8>> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT v FROM qqbot_cfg WHERE k = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.query_row((k.as_bytes(),), |r| r.get(0)).ok()
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
    // android watch + android pad
    const _MIXED_VER_INFO: ricq::version::Version = ricq::version::Version {
        apk_id: "com.tencent.mobileqq",
        app_id: 537118044,
        sub_app_id: 537118044,
        sort_version_name: "8.8.88.7083",
        build_ver: "8.8.88.7083",
        build_time: 1648004515,
        apk_sign: &[
            0xA6, 0xB7, 0x45, 0xBF, 0x24, 0xA2, 0xC2, 0x77, 0x52, 0x77, 0x16, 0xF6, 0xF3, 0x6E,
            0xB6, 0x8D,
        ],
        sdk_version: "6.0.0.2497",
        sso_version: 18,
        misc_bitmap: 150470524,
        sub_sig_map: 66560,
        main_sig_map: 16724722,
        protocol: Protocol::AndroidPad,
    };
    // {
    //     "apk_id": "com.tencent.mobileqq",
    //     "app_id": 537118044,
    //     "sub_app_id": 537118044,
    //     "app_key": "0S200MNJT807V3GE",
    //     "sort_version_name": "8.8.88.7083",
    //     "build_time": 1648004515,
    //     "apk_sign": "a6b745bf24a2c277527716f6f36eb68d",
    //     "sdk_version": "6.0.0.2497",
    //     "sso_version": 18,
    //     "misc_bitmap": 150470524,
    //     "main_sig_map": 16724722,
    //     "sub_sig_map": 66560,
    //     "dump_time": 1683193286,
    //     "protocol_type": 6
    //   }
    let client = Arc::new(Client::new(
        device,
        Protocol::AndroidWatch.into(),
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

async fn on_event(event: QEvent) {
    /* >>> recall_log
    #[allow(clippy::type_complexity)]
    static RECENT: Mutex<Vec<(i32, Vec<i32>, String, String, String)>> = Mutex::new(Vec::new());
    */
    match event {
        QEvent::GroupMessage(e) => {
            let e = e.inner;
            let msg = e.elements.to_string();
            // log!(INFO: "{msg}");
            if let Some(msg) = msg.strip_prefix('#') {
                let msg_parts = msg.split_whitespace().collect();
                if let Ok(reply) =
                    care!(commands::on_group_msg(e.group_code, msg_parts, &CLIENT).await)
                {
                    let reply = bot_msg(&reply);
                    let result = CLIENT.send_group_message(e.group_code, reply).await;
                    care!(result).ok();
                }
            }
            // log!("\n\x1b[93m[ksite]\x1b[0m {}", e.inner.elements);
            /* >>> recall_log
            let mut recent = RECENT.lock().unwrap();
            recent.push((e.time, e.seqs, e.group_name, e.group_card, msg));
            let len = recent.len();
            // size % 8 == 0, throttling while extreme scene
            if len >= 64 && len % 8 == 0 {
                // can't be recalled after 2 minutes
                recent.retain(|v| e.time - v.0 <= 120 + 5);
                // push_log!("cleaned {} expired messages", len - recent.len());
            }
            */
        }
        // the AndroidWatch protocol will not receive recall event
        /* >>> recall_log
        QEvent::GroupMessageRecall(e) => {
            let recent = RECENT.lock().unwrap();
            if let Some((_, _, group, user, content)) =
                recent.iter().find(|v| v.1.contains(&e.inner.msg_seq))
            {
                push_log!("recalled = {group} : {user} : {content}");
            }
        }
        */
        QEvent::Login(uin) => {
            push_log!("current account = {uin}");
        }
        _ => {}
    }
}

pub async fn notify(msg: &str) -> Result<()> {
    let msg_chain = bot_msg(msg);
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
        log!("units::qqbot received op {k}");
        match k {
            "set_device_json" => {
                db::cfg_set("device_json", &body);
            }
            "set_token_json" => {
                db::cfg_set("token_json", &body);
            }
            "set_notify_groups" => {
                db::cfg_set("notify_groups", &body);
            }
            _ => {
                log!(ERRO : "units::qqbot unknown op");
            }
        }
    }

    async fn get_handler() -> Html<String> {
        const PAGE: [&str; 2] = include_src!("page.html");
        let mut body = String::new();
        body += PAGE[0];
        for (time, content) in db::log_list().into_iter().rev() {
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

pub async fn tick() {
    use std::sync::Mutex;
    // ticker!(8, "XX:08:00", "XX:38:00");

    // db::log_clean();
    // Python is a programming language that lets you work quickly and integrate systems more effectively.

    async fn fake_notify(msg: &str) -> Result<()> {
        // log!(INFO: "fake_notify msg = {msg}");
        Ok(())
    }
    dbg!(
        fetch_text(str2req(
            "https://archlinux.org/packages/extra/x86_64/chromium/json/"
        ))
        .await
    );
    return;
    macro_rules! foo {
        ( $($y:expr),+ ) => {
            const fn ret_one<T>(_:&T)->i32{1}
            const fn ret_mutex<T>(_:&T)->Mutex<String>{Mutex::new(String::new())}
            const LEN: usize = -(-$( ret_one(&$y) )-+) as _;
            static LAST: [Mutex<String>; LEN] = [$( ret_mutex(&$y) ),+];
            async fn trigger2((name, desc, url): (&str, &str, &str), i: usize){
                let v = care!(fetch_text(str2req(url)).await, return);
                dbg!(&v);
                let v = v.rsplit_once(".nupkg").and_then(|v| v.0.rsplit_once('/')); // TODO
                let v = care!(v.e(), return).1;
                let mut last = LAST[i].lock().unwrap();
                if last.is_empty() {
                    *last = v.to_string();
                } else if *last != v {
                    *last = v.to_string();
                    drop(last); // avoid the mutex guard alive cross await point
                    care!(fake_notify(&format!("{name} {v} released!\n\n{desc}")).await, ());
                    // care!(notify(&v.to_lowercase()).await, ());
                }
            }
            let mut i = 0;
            tokio::join!($( trigger2($y, #[allow(unused_assignments)] { let ret = i; i += 1; ret }) ),+);
        };
    }
    foo!(
        ("Chrome", "Chrome is the official web browser from Google, built to be fast, secure, and customizable.", "https://archlinux.org/packages/extra/x86_64/chromium/json/"),
        ("Firefox", "The browsers that put your privacy first — and always have.", "https://archlinux.org/packages/extra/x86_64/firefox/json/"),
        ("VSCode", "Code editing. Redefined.", "https://archlinux.org/packages/extra/x86_64/code/json/"),
        ("Python", "Python is a programming language that lets you work quickly and integrate systems more effectively.", "https://archlinux.org/packages/core/x86_64/python/json/"),
        ("Rust", "A language empowering everyone to build reliable and efficient software.", "https://archlinux.org/packages/extra/x86_64/rust/json/"),
        ("Golang", "Build simple, secure, scalable systems with Go.", "https://archlinux.org/packages/extra/x86_64/go/json/")
    );
}

/*
https://archlinux.org/packages/?sort=&q=chromium&maintainer=&flagged=
https://api.winget.run/v2/packages/Google/Chrome
https://github.com/ScoopInstaller/Extras/blob/master/bucket/vscode.json
https://github.com/ScoopInstaller/Main/blob/master/bucket/rust.json
*/
