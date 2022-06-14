use crate::slot::slot;
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
        let break_marks = [
            "br>", "p>", "p ", "/p>", "div>", "div ", "/div>", "li>", "li ", "/li>",
        ];
        while let Some(v) = p.0.split_once('<') {
            p.0 = v.1.split_once('>').unwrap().1;
            let c = v.0.trim();
            if !c.is_empty() {
                o.push(c);
            }
            if *o.last().unwrap() != "<br>" {
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
        i = p.1.split_once("<link>").unwrap().1;
        p = i.split_once("</link>").unwrap();
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
}

type Res = ([(&'static str, &'static str); 2], Html<Vec<u8>>);

const PAGE: [&str; 2] = slot(include_str!("page.html"));

static CACHE: Lazy<Mutex<Res>> = Lazy::new(|| {
    let body = format!("{}<h2>Magazine is generating ...</h2>{}", PAGE[0], PAGE[1]);
    Mutex::new((
        [("cache-control", "no-cache"), ("refresh", "2")],
        Html(body.into_bytes()),
    ))
});

async fn refresh() {
    let mut o = vec![PAGE[0]];
    let g = |u| async move { reqwest::get(u).await.unwrap().text().await.unwrap() };
    let r = tokio::join!(
        g("https://rss.itggg.cn/zhihu/daily"),
        g("https://rss.itggg.cn/cnbeta"),
        g("https://rss.itggg.cn/oschina/news/industry"),
        g("https://rss.itggg.cn/1point3acres/post/hot")
    );
    generate(&r.0, &mut o, 20);
    generate(&r.1, &mut o, 20);
    generate(&r.2, &mut o, 20);
    generate(&r.3, &mut o, 10);
    o.push(PAGE[1]);

    let o = miniz_oxide::deflate::compress_to_vec(o.join("").as_bytes(), 10);
    *CACHE.lock().unwrap() = (
        [
            ("cache-control", "max-age=1800"),
            ("content-encoding", "deflate"),
        ],
        Html(o),
    );
}

pub fn service() -> Router {
    tokio::spawn(refresh());
    Router::new().route(
        "/magazine",
        MethodRouter::new().get(|| async { CACHE.lock().unwrap().clone() }),
    )
}

static TICKER: Lazy<Mutex<Ticker>> = Lazy::new(|| Mutex::new(Ticker::new_p8(&[(-1, 15, 0)])));
pub async fn tick() {
    if !TICKER.lock().unwrap().tick() {
        return;
    }

    refresh().await;
}
