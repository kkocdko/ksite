use crate::ticker::Ticker;
use axum::response::Html;
use axum::routing::MethodRouter;
use axum::Router;
use once_cell::sync::Lazy;
use std::sync::Mutex;

fn generate<'a>(mut i: &'a str, o: &mut Vec<&'a str>, mut limit: usize) {
    while let Some(mut p) = i.split_once("<item>") {
        limit -= 1;

        o.push("\n<details>\n");

        // title
        i = p.1.split_once("<![CDATA[").unwrap().1;
        p = i.split_once("]]>").unwrap();
        o.push("<summary>");
        o.push(p.0);
        o.push("</summary>\n");

        // content
        i = p.1.split_once("<![CDATA[").unwrap().1;
        p = i.split_once("]]>").unwrap();
        o.push("<section>");
        let list = [
            "p>", "p ", "/p>", "div>", "div ", "/div>", "li>", "li ", "/li>",
        ];
        while let Some(v) = p.0.split_once('<') {
            p.0 = v.1.split_once('>').unwrap().1;
            let c = v.0.trim();
            if !c.is_empty() {
                o.push(c);
            }
            if *o.last().unwrap() != "<br>" {
                for mark in list {
                    if v.1.starts_with(mark) {
                        o.push("<br>");
                        break;
                    }
                }
            }
        }
        o.push("</section>\n");

        // link
        i = p.1.split_once("<link>").unwrap().1;
        p = i.split_once("</link>").unwrap();
        o.push("<a href=\"");
        o.push(p.0);
        o.push("\">Original Link</a>\n");

        o.push("</details>\n");
        i = p.1;

        if limit == 0 {
            break;
        }
    }
}

type Res = ([(&'static str, &'static str); 1], Html<Vec<u8>>);

static PAGE: Lazy<(&str, &str)> =
    Lazy::new(|| include_str!("page.html").split_once("{main}").unwrap());

static CACHE: Lazy<Mutex<Res>> = Lazy::new(|| {
    let body = format!("{}<h2>Magazine is generating ...</h2>{}", PAGE.0, PAGE.1);
    Mutex::new(([("refresh", "2")], Html(body.into_bytes())))
});

async fn refresh() {
    let mut body = Vec::new();
    body.push(PAGE.0);
    let g = |u| async move { reqwest::get(u).await.unwrap().text().await.unwrap() };
    let r = tokio::join!(
        g("https://rsshub.app/zhihu/daily"),
        g("https://rsshub.app/cnbeta"),
        g("https://rsshub.app/oschina/news/industry")
    );
    generate(&r.0, &mut body, 20);
    generate(&r.1, &mut body, 20);
    generate(&r.2, &mut body, 20);
    body.push(PAGE.1);

    let mut buf = Vec::<u8>::new();
    brotli::BrotliCompress(&mut body.join("").as_bytes(), &mut buf, &Default::default()).unwrap();
    *CACHE.lock().unwrap() = ([("content-encoding", "br")], Html(buf));
}

pub fn service() -> Router {
    tokio::spawn(refresh());
    Router::new().route(
        "/magazine",
        MethodRouter::new().get(|| async { CACHE.lock().unwrap().clone() }),
    )
}

static TICKER: Lazy<Mutex<Ticker>> = Lazy::new(|| Mutex::new(Ticker::new_p8(&[(-1, 20, 0)])));
pub async fn tick() {
    if !TICKER.lock().unwrap().tick() {
        return;
    }
    refresh().await;
}
