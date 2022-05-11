/*
https://rsshub.app/zhihu/daily
https://rsshub.app/cnbeta
https://rsshub.app/oschina/news
*/

// into a brotli binary, refresh every hour.

use axum::http::HeaderValue;
// use crate::{db, ticker::Ticker};
// use axum::extract::Json;
use axum::http::header::IntoHeaderName;
use axum::response::Response;
use axum::response::{AppendHeaders, Html, IntoResponse};
use axum::routing::MethodRouter;
use axum::Router;
use once_cell::sync::Lazy;
use std::io::Read;
// use serde::Deserialize;
// use std::collections::HashMap;
use std::sync::Mutex;
// use std::time::SystemTime;

async fn fetch_text(url: &str) -> String {
    reqwest::get(url).await.unwrap().text().await.unwrap()
}


fn gen<'a>(mut r: &'a str, ret: &mut Vec<&'a str>, mut max: usize) {
    while let Some(mut p) = r.split_once("<item>") {
        max -= 1;

        ret.push("\n<details>\n");
        // title
        {
            r = p.1.split_once("<![CDATA[").unwrap().1;
            p = r.split_once("]]>").unwrap();
            ret.push("<summary>");
            ret.push(p.0);
            ret.push("</summary>\n");
        }
        // content
        {
            r = p.1.split_once("<![CDATA[").unwrap().1;
            p = r.split_once("]]>").unwrap();
            ret.push("<section>");

            while let Some(v) = p.0.split_once('<') {
                let i = v.0.trim();
                if !i.is_empty() {
                    ret.push(i);
                }
                if false
                    || v.1.starts_with("p>")
                    || v.1.starts_with("p ")
                    || v.1.starts_with("/p>")
                    || v.1.starts_with("div>")
                    || v.1.starts_with("div ")
                    || v.1.starts_with("/div>")
                    || v.1.starts_with("ol>")
                    || v.1.starts_with("ol ")
                    || v.1.starts_with("/ol>")
                    || v.1.starts_with("li>")
                    || v.1.starts_with("li ")
                    || v.1.starts_with("/li>")
                {
                    if *ret.last().unwrap() != "<br>" {
                        ret.push("<br>");
                    }
                }
                p.0 = v.1.split_once('>').unwrap().1;
            }

            ret.push("</section>\n");
        }
        // link
        {
            r = p.1.split_once("<link>").unwrap().1;
            p = r.split_once("</link>").unwrap();
            ret.push("<a href=\"");
            ret.push(p.0);
            ret.push("\">Original Link</a>\n");
        }
        ret.push("</details>\n");
        r = p.1;

        if max == 0 {
            break;
        }
    }
}

static CACHE: Lazy<Mutex<Vec<u8>>> = Lazy::new(|| Mutex::new(Vec::new()));

async fn refresh() {
    const PAGE: &str = include_str!("page.html");
    let mut to = Vec::new();
    let r = tokio::join!(
        fetch_text("https://rsshub.app/zhihu/daily"),
        fetch_text("https://rsshub.app/cnbeta"),
        fetch_text("https://rsshub.app/oschina/news") // TODO: replace this?
    );
    gen(&r.0, &mut to, 20);
    gen(&r.1, &mut to, 20);
    gen(&r.2, &mut to, 20);
    let raw = PAGE.replace("{main}", &to.join(""));
    let mut buf: Vec<u8> = Vec::new();
    brotli::BrotliCompress(&mut raw.as_bytes(), &mut buf, &Default::default());
    *CACHE.lock().unwrap() = buf;
}

async fn get_handler() -> impl IntoResponse {
    refresh().await;
    let binary = { CACHE.lock().unwrap().clone() };
    ([("content-encoding", "br")], Html(binary))
}

pub fn service() -> Router {
    Router::new().route("/magazine", MethodRouter::new().get(get_handler))
}
