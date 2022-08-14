//! Collections of my favorite news source.
use crate::care;
use crate::ticker::Ticker;
use crate::utils::{fetch_text, slot, OptionResult};
use anyhow::Result;
use axum::http::header::{HeaderName, CACHE_CONTROL, CONTENT_ENCODING, EXPIRES, REFRESH};
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use std::sync::Mutex; // with small data, Mutex seems faster than RwLock
use std::time::{Duration, SystemTime};

fn generate<'a>(mut i: &'a str, o: &mut Vec<&'a str>, mut limit: usize) -> Result<()> {
    while let Some(mut p) = i.split_once("<item>") {
        limit -= 1;

        o.push("\n<details>\n");

        // title
        i = p.1.split_once("<![CDATA[").e()?.1;
        p = i.split_once("]]>").e()?;
        o.push("<summary>");
        o.push(p.0);
        o.push("</summary>\n");

        // content
        i = p.1.split_once("<![CDATA[").e()?.1;
        p = i.split_once("]]>").e()?;
        o.push("<section>");
        let break_marks = [
            "br>", "p>", "p ", "/p>", "div>", "div ", "/div>", "li>", "li ", "/li>",
        ];
        while let Some(v) = p.0.split_once('<') {
            p.0 = v.1.split_once('>').e()?.1;
            let c = v.0.trim();
            if !c.is_empty() {
                o.push(c);
            }
            if *o.last().e()? != "<br>" {
                for mark in break_marks {
                    if v.1.starts_with(mark) {
                        o.push("<br>");
                        break;
                    }
                }
            }
        }
        o.push("</section>\n");

        // link
        i = p.1.split_once("<link>").e()?.1;
        p = i.split_once("</link>").e()?;
        o.push("<a href=\"");
        o.push(p.0);
        o.push("\">[ Original Link ]</a>\n");

        o.push("</details>\n");
        i = p.1;

        if limit == 0 {
            break;
        }
    }
    o.push("\n<br>\n");
    Ok(())
}

type Res = ([(HeaderName, String); 2], Html<Vec<u8>>);

const PAGE: [&str; 2] = slot(include_str!("page.html"));

static CACHE: Lazy<Mutex<Res>> = Lazy::new(|| {
    let body = format!("{}<h2>Magazine is generating ...</h2>{}", PAGE[0], PAGE[1]);
    Mutex::new((
        [(CACHE_CONTROL, "no-store".into()), (REFRESH, "2".into())],
        Html(body.into_bytes()),
    ))
});

async fn refresh() -> Result<()> {
    let mut o = vec![PAGE[0]];
    macro_rules! load {
        ( $( ($idx:tt, $url:expr) ),* $(,)? ) => {
            let r = tokio::join!( $( fetch_text($url), )* );
            let r = ($( r.$idx?, )*);
            $( generate(&r.$idx, &mut o, 20)?; )*
        };
    }
    load![
        (0, "https://rss.itggg.cn/zhihu/daily"),
        (1, "https://rss.itggg.cn/cnbeta"),
        (2, "https://rss.itggg.cn/oschina/news/industry"),
        (3, "https://rss.itggg.cn/1point3acres/post/hot3"),
    ];
    o.push(PAGE[1]);
    let o = miniz_oxide::deflate::compress_to_vec(o.join("").as_bytes(), 10);
    let expires = httpdate::fmt_http_date(SystemTime::now() + Duration::from_secs(3600));
    *CACHE.lock().unwrap() = (
        [(EXPIRES, expires), (CONTENT_ENCODING, "deflate".into())],
        Html(o),
    );
    Ok(())
}

pub fn service() -> Router {
    tokio::spawn(async { care!(refresh().await).ok() });
    Router::new().route(
        "/magazine",
        MethodRouter::new().get(|| async { CACHE.lock().unwrap().clone() }),
    )
}

static TICKER: Lazy<Ticker> = Lazy::new(|| Ticker::new_p8(&[(-1, 15, 0)]));
pub async fn tick() {
    if !TICKER.tick() {
        return;
    }

    care!(refresh().await).ok();
}
