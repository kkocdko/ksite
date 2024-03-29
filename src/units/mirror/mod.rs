//! Lazy mirror for caching linux distros' packages.

use crate::utils::LazyLock as Lazy;
use crate::utils::{str2req, with_retry, CLIENT};
use crate::{care, include_src, log, ticker};
use axum::body::HttpBody;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{MethodRouter, Router};
use std::fmt::Write as _;
use std::future::poll_fn;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

mod db {
    use crate::database::DB;
    use crate::strip_str;
    pub fn init() {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            CREATE TABLE IF NOT EXISTS mirror (path BLOB PRIMARY KEY, time INTEGER, finished INTEGER)
        "};
        let mut stmd = db.prepare(sql).unwrap();
        stmd.execute(()).unwrap();
    }
    pub fn get(path: &str) -> Option<(u64, bool)> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT rowid, finished FROM mirror WHERE path = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.query_row((path.as_bytes(),), |r| Ok((r.get(0)?, r.get(1)?)))
            .ok()
    }
    pub fn add(path: &str) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            INSERT INTO mirror VALUES (?, strftime('%s', 'now'), 0)
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((path.as_bytes(),)).unwrap();
    }
    pub fn set_finished(path: &str) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            UPDATE mirror SET finished = 1 WHERE path = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((path.as_bytes(),)).unwrap();
    }
    pub fn del(path: &str) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            DELETE FROM mirror WHERE path = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((path.as_bytes(),)).unwrap();
    }
    pub fn list() -> Vec<(u64, String, u64)> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT rowid, path, time FROM mirror
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.query_map((), |r| {
            Ok((r.get(0)?, String::from_utf8(r.get(1)?).unwrap(), r.get(1)?))
        })
        .unwrap()
        .map(|v| v.unwrap())
        .collect()
    }
}

fn gen_file_path(rowid: u64) -> PathBuf {
    static DIR: Lazy<String> = Lazy::new(|| {
        let mut dir = std::env::current_exe().unwrap();
        dir.set_extension("mirror");
        std::fs::create_dir(&dir).ok();
        dir.push("a"); // keep the slash ('/' or '\')
        let mut dir = dir.into_os_string().into_string().unwrap();
        dir.pop(); // the OsString is encoded by WTF-8, similer to UTF-8
        dir
    });
    let mut ret = DIR.clone();
    write!(&mut ret, "{rowid:0>20}").unwrap(); // string(2 ** 64).length == 20
    PathBuf::from(ret)
}

async fn handle(req_path: &str, target: String) -> Response {
    let fetch_target =
        || async { care!(with_retry(|| CLIENT.fetch(str2req(&target), None), 3, 500).await) };
    let db_get_result = db::get(req_path);
    if let Some((rowid, true)) = db_get_result {
        let file = File::open(gen_file_path(rowid)).await.unwrap();
        let reader_stream = tokio_util::io::ReaderStream::new(file);
        return axum::body::Body::from_stream(reader_stream).into_response();
    }
    // in progress, but not finished, just pipe it
    if db_get_result.is_some() {
        return match fetch_target().await {
            Ok(v) => v.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
    }
    db::add(req_path); // insert first to avoid condition race
    let mut body = match fetch_target().await {
        Ok(v) => v.into_body(),
        Err(e) => {
            log!(erro: "{e:?}");
            db::del(req_path);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let (tx, rx) = mpsc::channel(16);
    let req_path = req_path.to_owned();
    // even if the connection closed, the store process still running
    tokio::spawn(async move {
        let rowid = db::get(&req_path).unwrap().0;
        let file_path = gen_file_path(rowid);
        let mut file = File::create(&file_path).await.unwrap();
        while let Some(result) = poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await {
            match result {
                Ok(frame) if !frame.is_data() => {}
                Ok(frame) => {
                    let data = frame.into_data().unwrap();
                    file.write_all(&data).await.unwrap();
                    tx.send(Ok(data)).await.ok(); // ignore error if rx closed
                }
                Err(e) => {
                    log!(erro: "{e:?}");
                    let io_err_str = StatusCode::INTERNAL_SERVER_ERROR.as_str();
                    let io_err = std::io::Error::new(std::io::ErrorKind::Other, io_err_str);
                    tx.send(Err(io_err)).await.ok();
                    // remember to remove file and cache entry in database
                    file.set_len(0).await.unwrap();
                    drop(file);
                    tokio::fs::remove_file(&file_path).await.unwrap();
                    db::del(&req_path);
                    return;
                }
            }
        }
        db::set_finished(&req_path);
    });
    let rx_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    axum::body::Body::from_stream(rx_stream).into_response()
}

pub fn service() -> Router {
    db::init();
    Router::new()
        .route(
            "/mirror",
            MethodRouter::new().get(|| async {
                const PAGE: [&str; 2] = include_src!("page.html");
                let mut ret = String::new();
                ret += PAGE[0];
                let mut list = db::list();
                for (number, _, _) in &mut list {
                    *number = std::fs::metadata(gen_file_path(*number)).unwrap().len();
                }
                list.sort_by_key(|v| v.0);
                for (size, path, _) in list.into_iter().rev() {
                    // TODO: escape
                    writeln!(&mut ret, "{size: >12} {path}").unwrap();
                }
                ret += PAGE[1];
                Html(ret)
            }),
        )
        .route(
            "/mirror/*path",
            MethodRouter::new().get(|Path(p): Path<String>| async move {
                // http://mirror.nju.edu.cn/fedora/
                // http://mirrors.ustc.edu.cn/fedora/
                // http://mirrors.tuna.tsinghua.edu.cn/fedora/
                if let Some(r) = p.strip_prefix("fedora/") {
                    // return handle(&p, format!("http://mirrors.ustc.edu.cn/fedora/{r}")).await;
                    return handle(&p, format!("http://mirror.23m.com/fedora/linux/{r}")).await;
                }
                if let Some(r) = p.strip_prefix("ubuntu/") {
                    return handle(&p, format!("http://mirrors.ustc.edu.cn/ubuntu/{r}")).await;
                }
                if let Some(r) = p.strip_prefix("debian/") {
                    return handle(&p, format!("http://mirrors.ustc.edu.cn/debian/{r}")).await;
                }
                if let Some(r) = p.strip_prefix("debian-security/") {
                    return handle(
                        &p,
                        format!("http://mirrors.ustc.edu.cn/debian-security/{r}"),
                    )
                    .await;
                }
                StatusCode::NOT_FOUND.into_response()
            }),
        )
}

pub async fn tick() {
    ticker!(return, 8, "XX:14:00");

    tokio::task::spawn_blocking(|| {
        let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        for (id, req_path, time) in db::list() {
            if now - time < 3600 * 24 {
                continue;
            }
            let file_path = gen_file_path(id);
            std::fs::remove_file(file_path).unwrap();
            db::del(&req_path);
        }
    });
}
