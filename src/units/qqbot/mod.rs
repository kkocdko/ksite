//! QQ robot for fun.

use crate::auth::auth_layer;
use crate::units::admin;
use crate::utils::{elapse, fetch_json, fetch_text, log_escape, str2req, LazyLock, OptionResult};
use crate::{care, include_src, log, ticker};
use anyhow::Result;
use axum::body::Bytes;
use axum::middleware;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use rand::{thread_rng, Rng};
use ricq::client::NetworkStatus;
use ricq::handler::QEvent;
use ricq::msg::{MessageChain, MessageElem};
use ricq::{Client, Device, LoginResponse, Protocol, QRCodeState};
use std::collections::HashMap;
use std::io::Write as _;
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

static QR: Mutex<Bytes> = Mutex::new(Bytes::new());
static CLIENT: LazyLock<Arc<Client>> = LazyLock::new(|| {
    log!(INFO: "init client");
    let device = match admin::db::get("qqbot_device") {
        Some(v) => serde_json::from_slice(&v).unwrap(),
        None => {
            let device = Device::random();
            let device_json = serde_json::to_string(&device).unwrap();
            admin::db::set("qqbot_device", device_json.as_bytes());
            device
        }
    };
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
                    log!(WARN: "offline, fn start returned");
                } => {}
                _ = async {
                    tokio::time::sleep(Duration::from_millis(200)).await; // waiting for connected
                    care!(launch().await);
                    CLIENT.do_heartbeat().await;
                    log!(WARN: "offline, fn do_heartbeat returned");
                } => {}
            };
            CLIENT.stop(NetworkStatus::Unknown);
            let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
            if now - last < 30 {
                log!(WARN: "reconnection was stopped, overfrequency");
                return;
            }
            last = now;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    client
});

async fn launch() -> Result<()> {
    // # Tips about Login
    // 1. Run on local host, login by qrcode.
    // 2. Run on remote, copy device_json and token_json to database.
    // 3. Restart remote server.
    if let Some(v) = admin::db::get("qqbot_token") {
        let token = serde_json::from_slice(&v)?;
        CLIENT.token_login(token).await?;
        log!(INFO: "login by token");
    } else {
        let mut qr_resp = CLIENT.fetch_qrcode().await?;
        let mut img_sig = Bytes::new();
        loop {
            match qr_resp {
                QRCodeState::ImageFetch(inner) => {
                    log!(INFO: "qrcode fetched");
                    *QR.lock().unwrap() = inner.image_data;
                    img_sig = inner.sig;
                }
                QRCodeState::Timeout => {
                    log!(INFO: "qrcode timeout");
                    qr_resp = CLIENT.fetch_qrcode().await?;
                    continue;
                }
                QRCodeState::Confirmed(inner) => {
                    log!(INFO: "qrcode confirmed");
                    let login_resp = CLIENT
                        .qrcode_login(&inner.tmp_pwd, &inner.tmp_no_pic_sig, &inner.tgt_qr)
                        .await?;
                    if let LoginResponse::DeviceLockLogin { .. } = login_resp {
                        CLIENT.device_lock_login().await?;
                    }
                    log!(INFO: "login by qrcode");
                    break;
                }
                QRCodeState::WaitingForScan => log!(INFO: "qrcode waiting for scan"),
                QRCodeState::WaitingForConfirm => log!(INFO: "qrcode waiting for confirm"),
                QRCodeState::Canceled => log!(INFO: "qrcode canceled"),
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
    admin::db::set("qqbot_token", serde_json::to_string(&token)?.as_bytes());
    Ok(())
}

fn bot_msg(content: &str) -> MessageChain {
    MessageChain::new(ricq::msg::elem::Text::new(format!("[BOT] {content}")))
}

/// Generate reply from message parts
async fn on_group_msg(
    group_code: i64,
    msg_parts: Vec<&str>,
    client: &ricq::Client,
) -> Result<String> {
    static REPLIES: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(Default::default);
    Ok(match msg_parts[..] {
        ["帮助"] | ["help"] => "参阅 https://github.com/kkocdko/ksite/blob/main/src/units/qqbot/commands.rs#:~:text=match%20msg_parts".into(),
        ["运行平台"] => concat!(
            env!("CARGO_PKG_NAME"),
            " v",
            env!("CARGO_PKG_VERSION"),
            " with ricq and axum"
        )
        .into(),
        ["kk单身多久了"] => format!("kk已连续单身 {:.3} 天了", elapse(10485432e2)),
        // ["开学倒计时"] => format!("距 开学 仅 {:.3} 天", -elapse(16617312e2)),
        // ["高考倒计时"] => format!("距 2023 高考仅 {:.3} 天", -elapse(16860996e2)),
        ["驶向深蓝"] => {
            let url = "https://api.lovelive.tools/api/SweetNothings?genderType=M";
            fetch_text(str2req(url)).await?
        }
        ["吟诗"] => {
            let url = "https://v1.jinrishici.com/all.json";
            fetch_json(str2req(url), "/content").await?
        }
        // ["新闻"] => {
        //     let i = thread_rng().gen_range(3..20);
        //     let r = fetch_text("https://m.cnbeta.com/wap").await?;
        //     let r = r.split("htm\">").nth(i).e()?.split_once('<').e()?;
        //     r.0.into()
        // }
        ["rand", from, to] | ["随机数", from, to] => {
            let range = from.parse::<i64>()?..=to.parse()?;
            let v = thread_rng().gen_range(range);
            format!("{v} in range [{from},{to}]")
        }
        ["抽签", a, b] => {
            let v = thread_rng().gen_range(0..=1);
            format!("你抽中了 {}", [a, b][v])
        }
        ["btc"] | ["比特币"] => {
            let url = "https://chain.so/api/v2/get_info/BTC";
            let price = fetch_json(str2req(url), "/data/price").await?;
            format!("1 BTC = {} USD", price.trim_end_matches('0'))
        }
        ["eth"] | ["以太坊"] | ["以太币"] => {
            let url = "https://api.blockchair.com/ethereum/stats";
            let price = fetch_json(str2req(url), "/data/market_price_usd").await?;
            format!("1 ETH = {} USD", price.trim_end_matches('0'))
        }
        ["doge"] | ["狗狗币"] => {
            let url = "https://api.blockchair.com/dogecoin/stats";
            let price = fetch_json(str2req(url), "/data/market_price_usd").await?;
            format!("1 DOGE = {} USD", price.trim_end_matches('0'))
        }
        ["我有个朋友", name, "说", content] => {
            let mut message_chain = MessageChain::default();

            let mut rich_msg = MessageElem::RichMsg(Default::default());
            if let MessageElem::RichMsg(v) = &mut rich_msg {
                let body = format!(
                    r#"<msg serviceID="35" templateID="1" action="viewMultiMsg" brief="[聊天记录]" tSum="1" flag="3"><item layout="1"><title>群聊的聊天记录</title><title>{name}: {content}</title><hr/><summary>查看1条转发消息</summary></item></msg>"#
                );
                let mut encoder = ZlibEncoder::new(vec![1], Compression::none());
                encoder.write_all(body.as_bytes()).ok();
                v.template1 = Some(encoder.finish().unwrap());
                v.service_id = Some(35);
            }
            message_chain.0.push(rich_msg);

            let mut general_flags = MessageElem::GeneralFlags(Default::default());
            if let MessageElem::GeneralFlags(v) = &mut general_flags {
                v.pb_reserve = Some([120, 0, 248, 1, 0, 200, 2, 0].into());
                v.pendant_id = Some(0);
            }
            message_chain.0.push(general_flags);

            client.send_group_message(group_code, message_chain).await?;
            "你朋友确实是这么说的".into()
        }
        ["垃圾分类", i] => {
            let url = format!("https://api.muxiaoguo.cn/api/lajifl?m={i}");
            match fetch_json(str2req(url), "/data/type").await {
                Ok(v) => format!("{i} {v}"),
                Err(_) => format!("鬼知道 {i} 是什么垃圾呢"),
            }
        }
        ["聊天", i, ..] => {
            let url = format!("https://api.ownthink.com/bot?spoken={i}");
            fetch_json(str2req(url), "/data/info/text").await?
        }
        ["设置回复", k, v] => {
            REPLIES.lock().unwrap().insert(k.into(), v.into());
            "记住啦".into()
        }
        [k, ..] => match REPLIES.lock().unwrap().get(k) {
            Some(v) => v.clone(),
            None => "指令有误".into(),
        },
        [] => "你没有附加任何指令呢".into(),
    })
}

fn _judge_spam(msg: &str) -> bool {
    const LIST: &[&str] = &[
        "重要",
        "通知",
        "群",
        "后果自负",
        "二维码",
        "同学",
        "免费",
        "资料",
    ];
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
        QEvent::GroupMessage(e) => {
            let e = e.inner;
            let msg = e.elements.to_string();
            // log!(INFO: "{msg}");
            if let Some(msg) = msg.strip_prefix('/') {
                let msg_parts = msg.split_whitespace().collect();
                if let Ok(reply) = care!(on_group_msg(e.group_code, msg_parts, &CLIENT).await) {
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
            log!(INFO: "current account = {uin}");
        }
        _ => {}
    }
}

async fn notify(msg: &str) -> Result<()> {
    let msg_chain = bot_msg(msg);
    let groups = care!(admin::db::get("qqbot_notify_groups").e())?;
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
        .layer(middleware::from_fn(auth_layer))
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

async fn update_notify(
    last_ver: &'static Mutex<String>,
    fetch_uri: &'static str,
    trim_tag: fn(&str, &str) -> Option<String>,
    gen_msg: fn(String) -> String,
) -> Result<()> {
    let req = str2req(fetch_uri);
    let resolved = "20.205.243.166:443".to_string(); // https://api.github.com/meta
    let res = crate::utils::CLIENT.fetch(req, Some(resolved)).await?;
    let body = axum::body::to_bytes(axum::body::Body::new(res), usize::MAX).await?;
    let body = String::from_utf8(Vec::from(body))?;
    let mut ver = String::new();
    for (_, tag) in body
        .split(".tar.gz\"")
        .filter_map(|part| part.rsplit_once("/tags/"))
    {
        if let Some(ret) = trim_tag(tag, &ver) {
            ver = ret;
        }
    }
    if ver.is_empty() {
        return Err(anyhow::anyhow!("ver.is_empty()"));
    }
    let skip = {
        let mut last = last_ver.lock().unwrap();
        let skip = last.is_empty() || *last == ver;
        *last = ver.clone();
        skip
    };
    // dbg!(&gen_msg(ver));
    if !skip {
        notify(&gen_msg(ver)).await?;
    }
    Ok(())
}

pub async fn tick() {
    ticker!(8, "XX:08:00", "XX:38:00");
    // Golang https://github.com/golang/go/tags
    // LLVM https://github.com/llvm/llvm-project/tags
    /*
    update_notify(
        {
            static LAST_VER: Mutex<String> = Mutex::new(String::new());
            &LAST_VER
        },
        "https://github.com/golang/go/tags",
        |cur, prev| Some(cur.to_string()),
        |ver| format!("Golang {ver} released!\n\nBuild simple, secure, scalable systems with Go."),
    )
    .await;
    update_notify(
        {
            static LAST_VER: Mutex<String> = Mutex::new(String::new());
            &LAST_VER
        },
        "https://github.com/python/cpython/tags",
        |cur, prev| match () {
            _ if cur.contains(&['a', 'b', 'r']) => None,
            _ => Some(cur.split_at(1).1.to_string()),
        },
        |ver| format!("Python {ver} released!\n\nPython is a programming language that lets you work quickly and integrate systems more effectively."),
    )
    .await;
    update_notify(
        {
            static LAST_VER: Mutex<String> = Mutex::new(String::new());
            &LAST_VER
        },
        "https://github.com/nodejs/node/tags", // TODO: sort
        |cur, prev| {
            let cur = cur.split_at(1).1.to_string();
            let cur_major = cur.split_once('.')?.0.parse::<i32>().ok()?;
            let prev_major = prev.split_once('.')?.0.parse::<i32>().ok()?;
            match cur_major > prev_major {
                true => Some(cur),
                false => None,
            }
        },
        |ver| format!("Node.js {ver} released!\n\nNode.js is an open-source, cross-platform JavaScript runtime environment."),
    )
    .await;
    */
    update_notify(
        {
            static LAST_VER: Mutex<String> = Mutex::new(String::new());
            &LAST_VER
        },
        "https://github.com/microsoft/vscode/tags",
        |cur, prev| match prev.is_empty() {
            true => Some(cur.to_string()),
            false => None,
        },
        |ver| format!("VSCode {ver} released!\n\nCode editing. Redefined."),
    )
    .await;
    update_notify(
        {
            static LAST_VER: Mutex<String> = Mutex::new(String::new());
            &LAST_VER
        },
        "https://github.com/rust-lang/rust/tags",
        |cur, prev| match prev.is_empty() {
            true => Some(cur.to_string()),
            false => None,
        },
        |ver| format!("Rust {ver} released!\n\nA language empowering everyone to build reliable and efficient software."),
    )
    .await;
}
