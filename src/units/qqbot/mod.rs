//! QQ robot for fun.

use crate::auth::auth_layer;
use crate::units::admin;
use crate::utils::{block_on, fetch_json, fetch_text, str2req, LazyLock, OptionResult};
use crate::{care, log, ticker};
use anyhow::Result;
use axum::body::Bytes;
use axum::http::header::*;
use axum::http::Request;
use axum::middleware;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use rand::{thread_rng, Rng as _};
use ricq::handler::QEvent;
use ricq::msg::{MessageChain, MessageElem};
use ricq::{Client, LoginResponse, Protocol, QRCodeState};
use std::io::Write as _;
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

static QR: Mutex<Bytes> = Mutex::new(Bytes::new());
static CLIENT: LazyLock<Arc<Client>> = LazyLock::new(|| {
    log!(info: "init client");
    let device = block_on(admin::db::get("qqbot_device".to_owned())).unwrap_or_else(|| {
        let default_device = br#"{"display":"OPPO.WATCH.3.12345","product":"mywatch","device":"watchthird","board":"eomam","model":"OPPO Watch 3","finger_print":"oppo/watch/watchthird:12/eomam.200122.001/3713053:user/release-keys","boot_id":"c551a017-7b25-a29c-d017-f5669c99f3f6","proc_version":"Linux 5.4.0-54-generic-JT1rcT5R (android-build@oppo.com)","imei":"596383386086907","brand":"Oppo","bootloader":"U-boot","base_band":"","version":{"incremental":"5891938","release":"12","codename":"REL","sdk":31},"sim_info":"T-Mobile","os_type":"android","mac_address":"00:50:56:C0:00:09","ip_address":[10,0,1,3],"wifi_bssid":"00:50:56:C0:00:09","wifi_ssid":"mywifi","imsi_md5":[168,95,162,8,95,25,127,174,97,161,163,42,13,203,28,159],"android_id":"c307656af5d64cba","apn":"wifi","vendor_name":"ColorOS Watch","vendor_os_name":"ColorOS Watch"}"#; // or ricq::Device::random()
        let bytes = Bytes::from_static(default_device);
        block_on(admin::db::set("qqbot_device".to_owned(), bytes.to_owned()));
        bytes
    });
    let device = serde_json::from_slice(&device).unwrap();
    let client_ver = Protocol::AndroidWatch.into();
    let client = Arc::new(Client::new(device, client_ver, on_event as fn(_) -> _));
    tokio::spawn(async {
        let mut last = UNIX_EPOCH.elapsed().unwrap().as_secs();
        loop {
            let addr = "msfwifi.3g.qq.com:8080";
            let tcp_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
            tokio::select! {
                _ = async {
                    CLIENT.start(tcp_stream).await;
                    log!(warn: "offline, fn start returned");
                } => {}
                _ = async {
                    tokio::time::sleep(Duration::from_millis(200)).await; // waiting for connected
                    care!(launch().await);
                    CLIENT.do_heartbeat().await;
                    log!(warn: "offline, fn do_heartbeat returned");
                } => {}
            };
            CLIENT.stop(ricq::client::NetworkStatus::Unknown);
            let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
            if now - last < 30 {
                log!(warn: "reconnection was stopped, overfrequency");
                return;
            }
            last = now;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    client
});

async fn launch() -> Result<()> {
    // Login by qrcode locally, then copy qqbot_device and qqbot_token to remote and login by token
    if let Some(v) = admin::db::get("qqbot_token".to_owned()).await {
        let token = serde_json::from_slice(&v)?;
        CLIENT.token_login(token).await?;
        log!(info: "login by token");
    } else {
        let mut qr_resp = CLIENT.fetch_qrcode().await?;
        let mut img_sig = Bytes::new();
        loop {
            match qr_resp {
                QRCodeState::ImageFetch(inner) => {
                    log!(info: "qrcode fetched");
                    *QR.lock().unwrap() = inner.image_data;
                    img_sig = inner.sig;
                }
                QRCodeState::Timeout => {
                    log!(info: "qrcode timeout");
                    qr_resp = CLIENT.fetch_qrcode().await?;
                    continue;
                }
                QRCodeState::Confirmed(inner) => {
                    log!(info: "qrcode confirmed");
                    let login_resp = CLIENT
                        .qrcode_login(&inner.tmp_pwd, &inner.tmp_no_pic_sig, &inner.tgt_qr)
                        .await?;
                    if let LoginResponse::DeviceLockLogin { .. } = login_resp {
                        CLIENT.device_lock_login().await?;
                    }
                    log!(info: "login by qrcode");
                    break;
                }
                QRCodeState::WaitingForScan => log!(info: "qrcode waiting for scan"),
                QRCodeState::WaitingForConfirm => log!(info: "qrcode waiting for confirm"),
                QRCodeState::Canceled => log!(info: "qrcode canceled"),
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
    let qqbot_token = serde_json::to_string(&token)?;
    admin::db::set("qqbot_token".to_owned(), Bytes::from(qqbot_token)).await;
    Ok(())
}

fn bot_msg(content: &str) -> MessageChain {
    MessageChain::new(ricq::msg::elem::Text::new(format!("[BOT] {content}")))
}

/// Generate reply from message parts
async fn on_group_msg(group_code: i64, msg: String) -> Result<()> {
    let (head, rest) = msg.split_at(msg.find(' ').unwrap_or(msg.len()));
    let rest = rest.trim();
    /// (stamp secs) -> (days) , javascript: `new Date("2001.01.01 06:00").getTime() / 1e3`
    fn elapse(stamp: f64) -> f64 {
        let now = UNIX_EPOCH.elapsed().unwrap().as_secs_f64();
        (now - stamp) / 864e2 // unit: days
    }
    let msg_chain = match head {
        _ if rest.len() > 120 => bot_msg("请长话短说"),
        "/帮助" => {
            bot_msg("参阅 https://github.com/kkocdko/ksite/blob/main/src/units/qqbot/mod.rs#L132")
        }
        "/运行平台" => bot_msg(concat!(
            env!("CARGO_PKG_NAME"),
            " v",
            env!("CARGO_PKG_VERSION"),
            " with ricq and axum"
        )),
        "/kk单身多久了" => bot_msg(&format!("kk已连续单身 {:.3} 天了", elapse(10485432e2))),
        "/开学倒计时" => bot_msg(&format!("距 开学 仅 {:.3} 天", -elapse(17088768e2))), // 20240226 UTC+8
        "/吟诗" => bot_msg(&fetch_text(str2req("https://v1.jinrishici.com/all.txt")).await?),
        "/随机数" if rest.splitn(2, ' ').all(|v| v.parse::<u32>().is_ok()) => {
            let mut rest = rest.splitn(2, ' ');
            let from: u32 = rest.next().unwrap().parse().unwrap();
            let to: u32 = rest.next().unwrap().parse().unwrap();
            let v = thread_rng().gen_range(from..=to);
            bot_msg(&format!("{v} ~ [{from},{to}]"))
        }
        "/我有个朋友" if rest.splitn(3, ' ').count() == 3 => {
            let mut rest = rest.splitn(3, ' ');
            let name = rest.next().unwrap();
            rest.next().unwrap(); // == "说"
            let content = rest.next().unwrap();
            let mut rich_msg = MessageElem::RichMsg(Default::default());
            if let MessageElem::RichMsg(v) = &mut rich_msg {
                let body = format!(
                    r#"<msg serviceID="35" templateID="1" action="viewMultiMsg" brief="[聊天记录]" tSum="1" flag="3"><item layout="1"><title>群聊的聊天记录</title><title>{name}: {content}</title><hr/><summary>查看1条转发消息</summary></item></msg>"#
                );
                let level = flate2::Compression::none();
                let mut encoder = flate2::write::ZlibEncoder::new(vec![1], level);
                encoder.write_all(body.as_bytes()).ok();
                v.template1 = Some(encoder.finish().unwrap());
                v.service_id = Some(35);
            }
            let mut general_flags = MessageElem::GeneralFlags(Default::default());
            if let MessageElem::GeneralFlags(v) = &mut general_flags {
                v.pb_reserve = Some([120, 0, 248, 1, 0, 200, 2, 0].into());
                v.pendant_id = Some(0);
            }
            MessageChain(vec![rich_msg, general_flags])
        }
        "/聊天" => {
            let mut body = String::new();
            body += r#"{"stream":false,"model":"gpt-3.5-turbo","messages":[{"role":"system","content":"\nYou are kkGPT, a large language model trained by kkocdko.\nYour reply must be less than 70 words.\n\n"},{"role":"user","content":"#;
            body += &serde_json::to_string(rest).unwrap();
            body += r#"}]}"#;
            #[rustfmt::skip]
            let req = Request::post(concat!("https://www.gp", "tapi.us/v1/ch", "at/com", "pletions"))
                .header(HOST, concat!("www.gp", "tapi.us"))
                .header(CONTENT_TYPE, "application/json")
                .header(AUTHORIZATION, concat!("Bea", "rer s", "k-e", "s5Zonw3CrWjGUdrEaF", "eF428E1F449D4AcCd8a19Fa1d854c"))
                .body(axum::body::Body::from(body))
                .unwrap();
            bot_msg(&fetch_json(req, "/choices/0/message/content").await?)
        }
        _ => bot_msg("指令有误"),
    };
    CLIENT.send_group_message(group_code, msg_chain).await?;
    Ok(())
}

fn _judge_spam(msg: &str) -> bool {
    const LIST: &[&str] = &["重要", "通知", "群", "后果自负", "二维码", "同学", "免费"];
    const SENSITIVITY: f64 = 0.7;
    fn judge(msg: &str, list: &[&str], sensitivity: f64) -> bool {
        let len: usize = list.len();
        let expect = ((1.0 - sensitivity) * (len as f64)) as usize;
        let mut matched = 0;
        for (i, entry) in list.iter().enumerate() {
            if msg.contains(entry) {
                matched += 1;
            }
            if matched > expect {
                return true;
            } else if len - i - 1 + matched <= expect {
                return false;
            }
        }
        false
    }
    judge(msg, LIST, SENSITIVITY)
}

async fn on_event(event: QEvent) {
    /* >>> recall_log
    #[allow(clippy::type_complexity)]
    static RECENT: Mutex<Vec<(i32, Vec<i32>, String, String, String)>> = Mutex::new(Vec::new());
    */
    match event {
        QEvent::GroupMessage(mut e) => {
            // println!("\n\n===== e.inner.elements.0 =====");
            // for v in &e.inner.elements.0 {
            //     println!(">>> {:?}", v);
            // }
            for el in e.inner.elements.0 {
                if let MessageElem::Text(mut v) = el {
                    if let Some(v) = v.str.take() {
                        if v.starts_with('/') {
                            care!(on_group_msg(e.inner.group_code, v).await);
                        }
                    }
                    break;
                }
            }
            // log!(info: "\n\x1b[93m[qq]\x1b[0m {}", e.inner.elements.to_string());
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
            log!(info: "current account = {uin}");
        }
        _ => {}
    }
}

async fn notify(msg: &str) -> Result<()> {
    let msg_chain = bot_msg(msg);
    let groups = care!(admin::db::get("qqbot_notify_groups".to_owned()).await.e())?;
    let groups = care!(serde_json::from_slice::<Vec<i64>>(&groups))?;
    for group in groups {
        CLIENT.send_group_message(group, msg_chain.clone()).await?;
    }
    Ok(())
}

pub fn service() -> Router {
    CLIENT.get_status(); // init client
    Router::new()
        .route(
            "/qqbot",
            MethodRouter::new().get(Html(
                "<!DOCTYPE html><html style='color-scheme:light dark'><img src='/qqbot/qr'></html>",
            )),
        )
        .route(
            "/qqbot/qr",
            MethodRouter::new().get(|| async { QR.lock().unwrap().to_owned() }),
        )
        .route_layer(middleware::from_fn(auth_layer))
    // tokio::spawn(async {
    //     let on_event = |mut event: QEvent| async { event = dbg!(event) }; // interesting noop
    //     let device = Device::random();
    //     let client_ver = Protocol::AndroidWatch.into();
    //     let client = Arc::new(Client::new(device, client_ver, on_event as fn(_) -> _));
    //     let addr = "msfwifi.3g.qq.com:8080";
    //     let tcp_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    //     // let tcp_stream = DefaultConnector.connect(&client).await.unwrap();
    //     let client_clone = client.clone();
    //     tokio::spawn(async move {
    //         client_clone.start(tcp_stream).await;
    //     });
    //     tokio::time::sleep(Duration::from_millis(200)).await;
    //     dbg!(client.get_status());
    //     let mut qr_resp = client.fetch_qrcode().await.unwrap();
    //     dbg!(qr_resp);
    // });
}

pub async fn tick() {
    ticker!(return, 8, "XX:08:00", "XX:38:00");
    async fn update_notify(
        last_ver: &'static Mutex<String>,
        fetch_uri: &'static str,
        proc_tag: fn(Option<String>, &str) -> Option<String>,
        gen_msg: fn(String) -> String,
    ) -> Result<()> {
        use crate::utils::CLIENT_NO_SNI;
        let req = str2req(fetch_uri);
        let resolved = "20.200.245.247:443".to_string(); // https://api.github.com/meta | https://ip.tool.chinaz.com/20.200.245.245 | time curl --insecure -H "Host: github.com" https://20.200.245.247:443/nodejs/node/tags > /dev/null
        let res = CLIENT_NO_SNI.fetch(req, Some(resolved)).await?;
        let body = axum::body::to_bytes(axum::body::Body::new(res), usize::MAX).await?;
        let body = String::from_utf8(Vec::from(body))?;
        let mut ver = body
            .split(".tar.gz\"")
            .filter_map(|part| Some(part.rsplit_once("/tags/")?.1))
            .fold(None, proc_tag);
        let ver = match ver {
            Some(v) => v,
            None => return Err(anyhow::anyhow!("ver.is_empty()")),
        };
        let skip = {
            let mut last = last_ver.lock().unwrap();
            let skip = last.is_empty() || *last == ver;
            *last = ver.clone();
            skip
        };
        // println!("{fetch_uri} {ver}");
        if !skip {
            notify(&gen_msg(ver)).await?;
        }
        Ok(())
    }
    fn smart_tag(prev: Option<String>, cur: &str) -> Option<String> {
        let cur = cur.trim_start_matches(|c: char| !c.is_ascii_digit());
        if cur.contains(['a', 'b', 'r']) {
            return prev;
        }
        if prev.is_none() {
            return Some(cur.to_owned());
        }
        let cur_major = cur.split_once('.')?.0.parse::<i32>().ok()?;
        let prev_major = prev.as_ref()?.split_once('.')?.0.parse::<i32>().ok()?;
        match cur_major > prev_major {
            true => Some(cur.to_owned()),
            false => prev,
        }
    };
    macro_rules! make_ver_store {
        () => {{
            static LAST_VER: Mutex<String> = Mutex::new(String::new());
            &LAST_VER
        }};
    }
    #[rustfmt::skip]
    tokio::join!(
    update_notify(
        make_ver_store!(),
        "https://github.com/golang/go/tags",
        smart_tag,
        |ver| format!("Golang {ver} released!\n\nBuild simple, secure, scalable systems with Go."),
    ),
    update_notify(
        make_ver_store!(),
        "https://github.com/python/cpython/tags",
        smart_tag,
        |ver| format!("Python {ver} released!\n\nPython is a programming language that lets you work quickly and integrate systems more effectively."),
    ),
    update_notify(
        make_ver_store!(),
        "https://github.com/nodejs/node/tags",
        smart_tag,
        |ver| format!("Node.js {ver} released!\n\nNode.js is an open-source, cross-platform JavaScript runtime environment."),
    ),
    update_notify(
        make_ver_store!(),
        "https://github.com/microsoft/vscode/tags",
        |prev, cur| prev.or_else(|| Some(cur.to_owned())),
        |ver| format!("VSCode {ver} released!\n\nCode editing. Redefined."),
    ),
    update_notify(
        make_ver_store!(),
        "https://github.com/rust-lang/rust/tags",
        |prev, cur| prev.or_else(|| Some(cur.to_owned())),
        |ver| format!("Rust {ver} released!\n\nA language empowering everyone to build reliable and efficient software."),
    ),
    );
    // joinset     6059432 bytes
    // join macro  6043496 bytes
}
