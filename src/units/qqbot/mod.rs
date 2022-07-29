mod base;
use crate::care;
use crate::ticker::Ticker;
use crate::utils::{elapse, fetch_json, fetch_text, OptionResult};
use anyhow::Result;
use axum::routing::{MethodRouter, Router};
use base::{db_groups_insert, get_handler, get_login_qr, notify, post_handler};
use once_cell::sync::Lazy;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use tokio::sync::Mutex;

/// generate reply from message parts
async fn gen_reply(msg: Vec<&str>) -> Result<String> {
    static REPLIES: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
        Mutex::new(HashMap::from([
            ("呜".into(), "呜".into()),
            ("你说对吧".into(), "啊对对对".into()),
            ("运行平台".into(), "ksite / axum / ricq".into()),
        ]))
    });
    Ok(match msg[..] {
        ["kk单身多久了"] => format!("kk已连续单身 {:.3} 天了", elapse(10485432e2)),
        // ["开学倒计时"] => format!("距 开学 仅 {:.3} 天", -elapse(16561728e2)),
        ["高考倒计时"] => format!("距 2023 高考仅 {:.3} 天", -elapse(16860996e2)),
        ["驶向深蓝"] => {
            let url = "https://api.lovelive.tools/api/SweetNothings?genderType=M";
            fetch_text(url).await?
        }
        ["吟诗"] => {
            let url = "https://v1.jinrishici.com/all.json";
            fetch_json(url, "/content").await?
        }
        ["新闻"] => {
            let i = thread_rng().gen_range(3..20);
            let r = fetch_text("https://m.cnbeta.com/wap").await?;
            let r = r.split("htm\">").nth(i).e()?.split_once('<').e()?;
            r.0.into()
        }
        ["RAND", from, to] | ["随机数", from, to] => {
            let range = from.parse::<i64>()?..=to.parse()?;
            let v = thread_rng().gen_range(range);
            format!("{v} in range [{from},{to}]")
        }
        ["BTC"] | ["比特币"] => {
            let url = "https://chain.so/api/v2/get_info/BTC";
            let price = fetch_json(url, "/data/price").await?;
            format!("1 BTC = {} USD", price.trim_end_matches('0'))
        }
        ["ETH"] | ["以太坊"] | ["以太币"] => {
            let url = "https://api.blockchair.com/ethereum/stats";
            let price = fetch_json(url, "/data/market_price_usd").await?;
            format!("1 ETH = {} USD", price.trim_end_matches('0'))
        }
        ["DOGE"] | ["狗狗币"] => {
            let url = "https://api.blockchair.com/dogecoin/stats";
            let price = fetch_json(url, "/data/market_price_usd").await?;
            format!("1 DOGE = {} USD", price.trim_end_matches('0'))
        }
        ["垃圾分类", i] => {
            let url = format!("https://api.muxiaoguo.cn/api/lajifl?m={i}");
            match fetch_json(&url, "/data/type").await {
                Ok(v) => format!("{i} {v}"),
                Err(_) => format!("鬼知道 {i} 是什么垃圾呢"),
            }
        }
        ["聊天", i, ..] => {
            let url = format!("https://api.ownthink.com/bot?spoken={i}");
            fetch_json(&url, "/data/info/text").await?
        }
        ["订阅通知", v] => {
            db_groups_insert(v.parse()?);
            format!("已为群 {v} 订阅通知")
        }
        ["设置回复", k, v] => {
            REPLIES.lock().await.insert(k.into(), v.into());
            "记住啦".into()
        }
        [k, ..] => match REPLIES.lock().await.get(k) {
            Some(v) => v.clone(),
            None => "指令有误".into(),
        },
        [] => "你没有附加任何指令呢".into(),
    })
}

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

#[allow(unused)]
fn judge_spam(msg: &str) -> bool {
    let sensitivity = 0.7;
    let list = ["重要", "通知", "群", "后果自负", "二维码", "同学"];
    judge(msg, &list, sensitivity)
}

pub fn service() -> Router {
    get_login_qr(); // init client
    Router::new()
        .route(
            "/qqbot",
            MethodRouter::new().get(get_handler).post(post_handler),
        )
        .route(
            "/qqbot/qr",
            MethodRouter::new().get(|| async { get_login_qr() }),
        )
        .layer(crate::auth::auth_layer())
}

struct UpNotify {
    name: String,
    query_url: String,
    last: Mutex<String>,
}

impl UpNotify {
    async fn query(&self) -> Result<String> {
        let ret = fetch_text(&self.query_url).await?;
        let ret = ret.rsplit_once(".nupkg").e()?.0.rsplit_once('/').e()?.1;
        Ok(ret.split_once('.').e()?.1.to_string())
    }

    async fn trigger(&self) {
        let v = care!(self.query().await, return);
        let mut last = self.last.lock().await;
        if !last.is_empty() && *last != v {
            // store the latest value regardless of whether the notify succeeds or not
            care!(notify(format!("{} {v} released!", self.name)).await).ok();
        }
        *last = v;
    }

    fn new(name: &str, pkg_id: &str) -> Self {
        let query_url = format!("https://community.chocolatey.org/api/v2/package/{pkg_id}");
        Self {
            name: name.to_string(),
            query_url,
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
        // needless to spawn
        UP_CHROME.trigger(),
        UP_VSCODE.trigger(),
        UP_RUST.trigger()
    );
}
