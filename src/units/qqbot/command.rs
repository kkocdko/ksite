use crate::utils::{elapse, fetch_json, fetch_text};
use anyhow::Result;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use once_cell::sync::Lazy;
use rand::{thread_rng, Rng};
use ricq::msg::{MessageChain, MessageElem};
use std::collections::HashMap;
use std::io::Write as _;
use std::sync::Mutex;

/// Generate reply from message parts
pub async fn on_group_msg(
    group_code: i64,
    msg_parts: Vec<&str>,
    client: &ricq::Client,
) -> Result<String> {
    static REPLIES: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
        Mutex::new(HashMap::from([(
            "运行平台".into(),
            concat!(
                env!("CARGO_PKG_NAME"),
                " v",
                env!("CARGO_PKG_VERSION"),
                " with ricq and axum"
            )
            .into(),
        )]))
    });
    Ok(match msg_parts[..] {
        ["kk单身多久了"] => format!("kk已连续单身 {:.3} 天了", elapse(10485432e2)),
        // ["开学倒计时"] => format!("距 开学 仅 {:.3} 天", -elapse(16617312e2)),
        // ["高考倒计时"] => format!("距 2023 高考仅 {:.3} 天", -elapse(16860996e2)),
        ["驶向深蓝"] => {
            let url = "https://api.lovelive.tools/api/SweetNothings?genderType=M";
            fetch_text(url).await?
        }
        ["吟诗"] => {
            let url = "https://v1.jinrishici.com/all.json";
            fetch_json(url, "/content").await?
        }
        // ["新闻"] => {
        //     let i = thread_rng().gen_range(3..20);
        //     let r = fetch_text("https://m.cnbeta.com/wap").await?;
        //     let r = r.split("htm\">").nth(i).e()?.split_once('<').e()?;
        //     r.0.into()
        // }
        ["RAND", from, to] | ["随机数", from, to] => {
            let range = from.parse::<i64>()?..=to.parse()?;
            let v = thread_rng().gen_range(range);
            format!("{v} in range [{from},{to}]")
        }
        ["抽签", a, b] => {
            let v = thread_rng().gen_range(0..=1);
            format!("你抽中了 {}", [a, b][v])
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
            match fetch_json(&url, "/data/type").await {
                Ok(v) => format!("{i} {v}"),
                Err(_) => format!("鬼知道 {i} 是什么垃圾呢"),
            }
        }
        ["聊天", i, ..] => {
            let url = format!("https://api.ownthink.com/bot?spoken={i}");
            fetch_json(&url, "/data/info/text").await?
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
