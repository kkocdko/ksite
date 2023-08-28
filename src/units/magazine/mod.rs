//! Collections of my favorite news source.

use crate::utils::{fetch_text, str2req, OptionResult};
use crate::{care, include_src, log, ticker};
use anyhow::Result;
use axum::body::Bytes;
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::header::{CACHE_CONTROL, EXPIRES, REFRESH};
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

fn generate(mut i: &str, o: &mut String) -> Result<()> {
    // let mut limit = 20;
    while let Some(mut p) = i.split_once("<item>") {
        // limit -= 1;

        *o += "\n<details>\n";

        // title
        i = p.1.split_once("<![CDATA[").e()?.1;
        p = i.split_once("]]>").e()?;
        *o += "<summary>";
        *o += p.0;
        *o += "</summary>\n";

        // content
        i = p.1.split_once("<![CDATA[").e()?.1;
        p = i.split_once("]]>").e()?;
        *o += "<section>";
        let break_marks = [
            "br>", "p>", "p ", "/p>", "div>", "div ", "/div>", "li>", "li ", "/li>",
        ];
        while let Some(v) = p.0.split_once('<') {
            p.0 = v.1.split_once('>').e()?.1;
            let c = v.0.trim();
            if !c.is_empty() {
                *o += c;
            }
            if !o.ends_with("<br>") {
                for mark in break_marks {
                    if v.1.starts_with(mark) {
                        *o += "<br>";
                        break;
                    }
                }
            }
        }
        *o += "</section>\n";

        // link
        i = p.1.split_once("<link>").e()?.1;
        p = i.split_once("</link>").e()?;
        *o += "<a href=\"";
        *o += p.0;
        *o += "\">[ Original Link ]</a>\n";

        *o += "</details>\n";
        i = p.1;

        // if limit == 0 {
        //     break;
        // }
    }
    *o += "\n<br>\n";
    Ok(())
}

type Res = ([(HeaderName, HeaderValue); 2], Html<Bytes>);

const PAGE: [&str; 2] = include_src!("page.html");

static CACHE: Lazy<Mutex<Res>> = Lazy::new(|| {
    let body = format!("{}<h2>Magazine is generating ...</h2>{}", PAGE[0], PAGE[1]);
    // with small data, Mutex seems faster than RwLock
    Mutex::new((
        [
            (CACHE_CONTROL, HeaderValue::from_static("no-store")),
            (REFRESH, HeaderValue::from_static("2")),
        ],
        Html(Bytes::from(body)),
    ))
});

async fn refresh() -> Result<()> {
    let expires = httpdate::fmt_http_date(SystemTime::now() + Duration::from_secs(3600));
    async fn rss(p: &str) -> Result<String, ()> {
        let prefixs = [
            "https://rsshub.moeyy.cn",
            "https://rsshub.rssforever.com",
            "http://rsshub.uneasy.win",
            "https://rsshub.feeded.xyz",
            "https://rsshub.app",
        ];
        for prefix in prefixs {
            let req = str2req(prefix.to_string() + p);
            match fetch_text(req).await {
                Ok(v) if v.starts_with("<?xml") => return Ok(v),
                _ => {}
            };
        }
        log!(ERRO : "magazine fetch failed, p = {p}");
        Err(())
    }
    // tokio::task::JoinSe
    let r = tokio::join!(
        rss("/bbc?limit=5"),
        rss("/hackernews?limit=5"), // &mode=fulltext
        rss("/zhihu/daily?limit=7"),
        rss("/oschina/news/industry?limit=7"),
        rss("/1point3acres/post/hot3?limit=7"),
        rss("/rustcc/jobs?limit=4"),
    );
    let mut o = String::new();
    o += PAGE[0];
    o += &httpdate::fmt_http_date(SystemTime::now());
    o += "<br><br>";
    // let o = tokio::task::spawn_blocking(move || {
    r.0.map(|v| generate(&v, &mut o)).ok();
    r.1.map(|v| generate(&v, &mut o)).ok();
    r.2.map(|v| generate(&v, &mut o)).ok();
    r.3.map(|v| generate(&v, &mut o)).ok();
    r.4.map(|v| generate(&v, &mut o)).ok();
    r.5.map(|v| generate(&v, &mut o)).ok();
    // care!(r.6.map(|v| generate(&v, &mut o))).ok();
    o += PAGE[1];
    // use std::io::Write as _;
    // let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    // enc.write_all(o.as_bytes()).unwrap();
    // enc.finish().unwrap()
    // })
    // .await?;
    *CACHE.lock().unwrap() = (
        [
            (
                EXPIRES,
                HeaderValue::from_maybe_shared(Bytes::from(expires)).unwrap(),
            ),
            (
                HeaderName::from_static("server-timing"),
                HeaderValue::from_static("missedCache"),
            ),
            // (CONTENT_ENCODING, HeaderValue::from_static("gzip")),
        ],
        Html(Bytes::from(o)), // `bytes::Bytes` is cheaper than `Vec<u8>` on clone
    );
    Ok(())
}

pub fn service() -> Router {
    tokio::spawn(async {
        care!(refresh().await).ok();
    });
    Router::new().route(
        "/magazine",
        MethodRouter::new().get(|| async {
            CACHE.lock().unwrap().to_owned() // just clone some AtomicPtr inner
        }),
    )
}

pub async fn tick() {
    ticker!(8, "XX:04:00");

    care!(refresh().await).ok();
}
