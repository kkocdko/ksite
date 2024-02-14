//! Do v2ex.com daily sign-in.

use crate::units::admin;
use crate::utils::{with_retry, LazyLock, OptionResult, CLIENT_NO_SNI};
use crate::{care, include_src, log, ticker};
use anyhow::Result;
use axum::body::{Body, Bytes};
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::header::{ACCEPT, ACCEPT_LANGUAGE, COOKIE, HOST, REFERER, USER_AGENT};
use axum::http::Request;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

async fn do_mission(cookie: &str) -> Result<()> {
    log!(info: "v2exdaily::do_mission()");
    async fn fetch_authed(path: &str, cookie: &str) -> Result<String> {
        let uri = format!("https://fast.v2ex.com{path}");
        let mut req = Request::get(uri)
            .header(HOST, "fast.v2ex.com")
            .header(COOKIE, cookie)
            .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
            .header(ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8")
            .header(REFERER, "https://fast.v2ex.com/mission/daily")
            .header(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0")
            .body(Body::empty())?;
        let resolved = "104.20.9.218:443".to_string(); // https://www.nslookup.io/domains/fast.v2ex.com/dns-records/
        let res = CLIENT_NO_SNI.fetch(req, Some(resolved)).await?;
        let body = axum::body::to_bytes(axum::body::Body::new(res), usize::MAX).await?;
        Ok(String::from_utf8(Vec::from(body))?)
    }
    let page = fetch_authed("/mission/daily", cookie).await?;
    let needle = "/mission/daily/redeem?once=";
    let needle_idx = page.find(needle).e()? + needle.len();
    let code: Vec<_> = page
        .into_bytes()
        .into_iter()
        .skip(needle_idx)
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let code = std::str::from_utf8(&code)?;
    log!(info: "v2exdaily::do_mission() code = {code}");
    let _ret_page = fetch_authed(&format!("{needle}{code}"), cookie).await?;
    Ok(())
}

pub async fn tick() {
    ticker!(return, 8, "08:14:00");
    let cookies = care!(admin::db::get("v2ex_cookies".to_owned()).await.e(), return); // open F12, copy as nodejs fetch
    let cookies = care!(serde_json::from_slice::<Vec<String>>(&cookies), return);
    for cookie in cookies {
        care!(with_retry(|| do_mission(&cookie), 3, 2000).await, continue);
    }
}
