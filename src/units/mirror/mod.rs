//! Lazy mirror for caching linux distros' packages.

use crate::utils::{fetch, FileResponse, MpscResponse};
use crate::{care, db, include_src};
use anyhow::Result;
use axum::body::HttpBody;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use std::fmt::Write as _;
use std::future::Future;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

const STATE_PENDING: u8 = 1;
const STATE_FINISHED: u8 = 3;

fn db_init() {
    db!("CREATE TABLE IF NOT EXISTS mirror (path BLOB PRIMARY KEY, time INTEGER, state INTEGER)")
        .unwrap();
}

fn db_get(path: &str) -> Option<(u64, u8)> {
    db!(
        "SELECT rowid, state FROM mirror WHERE path = ?1",
        [path.as_bytes()],
        *|r| Ok((r.get(0)?, r.get(1)?))
    )
    .ok()
}

fn db_add(path: &str) {
    db!(
        "INSERT INTO mirror VALUES (?1, strftime('%s', 'now'), ?2)",
        [path.as_bytes(), STATE_PENDING]
    )
    .unwrap();
}

fn db_set(path: &str, state: u8) {
    db!(
        "UPDATE mirror SET state = ?2 WHERE path = ?1",
        [path.as_bytes(), state]
    )
    .unwrap();
}

fn db_del(path: &str) {
    db!("DELETE FROM mirror WHERE path = ?", [path.as_bytes()]).unwrap();
}

// TODO: clean outdated cache

fn gen_file_path(path: &str, rowid: u64) -> PathBuf {
    static DIR: Lazy<String> = Lazy::new(|| {
        let mut dir = std::env::current_exe().unwrap();
        dir.set_extension("mirror");
        std::fs::create_dir(&dir).ok();
        // keep the slash ('/' or '\')
        dir.push("a");
        let mut dir = dir.into_os_string().into_string().unwrap();
        dir.pop();
        dir
    });
    let mut ret = DIR.clone();
    let mut last_is_hyphen = true; // we dont't want a hyphen prefix
    let max_len = ret.len() + 60;
    for ch in path.chars() {
        if ret.len() > max_len {
            break;
        }
        if ch.is_ascii_alphanumeric() {
            ret.push(ch.to_ascii_lowercase());
            last_is_hyphen = false;
        } else if !last_is_hyphen {
            ret.push('-');
            last_is_hyphen = true;
        }
    }
    if !last_is_hyphen {
        ret.push('-');
    }
    write!(&mut ret, "{rowid}").unwrap();
    PathBuf::from(ret)
}

async fn retry<T, E: std::fmt::Debug, FUT: Future<Output = Result<T, E>>>(
    f: impl Fn() -> FUT,
) -> Result<T, E> {
    let mut err = None;
    for _ in 0..3 {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => err = Some(e),
        };
        tokio::time::sleep(Duration::from_millis(700)).await;
    }
    Err(err.unwrap())
}

async fn handle(path: &str, relative: &str, target: &str) -> Response {
    let url = || target.to_string() + relative;
    match db_get(path) {
        Some((rowid, state)) if state == STATE_FINISHED => {
            // println!("m:hit  {path}");
            // let file = match File::open(gen_file_path(path, rowid)).await {
            //     Ok(v) => v,
            //     Err(e) => {
            //         dbg!(gen_file_path(path, rowid));
            //         std::process::exit(0);
            //     }
            // };
            let file = File::open(gen_file_path(path, rowid)).await.unwrap();
            return FileResponse::new(file).into_response();
        }
        Some(_) => {
            // println!("m:pen  {path}");
            let u = url();
            return match care!(retry(|| fetch(&u)).await) {
                Ok(v) => v.into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };
        }
        None => {}
    };
    // println!("m:mis  {path}");
    db_add(path);
    let rowid = db_get(path).unwrap().0;
    let res_url = url();
    // In most cases the Internet will be slower than LANs and filesystem. However, if the Internet is too fast, it can lead to data droping if the mpsc capacity bound is reached.
    let (tx, rx) = mpsc::channel(16);
    // even if the connection closed, the store process still running
    let path = path.to_owned();
    let file_path = gen_file_path(&path, rowid);
    tokio::spawn(async move {
        let result = retry(|| async {
            let response = fetch(&res_url).await?;
            let mut file = File::create(&file_path).await.unwrap();
            let mut body = response.into_body();
            while let Some(result) = body.data().await {
                match result {
                    Ok(buf) => {
                        file.write_all(&buf).await.unwrap();
                        tx.send(Ok(buf)).await.ok(); // ignore error if rx closed
                    }
                    Err(e) => {
                        file.set_len(0).await.unwrap();
                        drop(file);
                        tokio::fs::remove_file(&file_path).await.unwrap();
                        let io_err = io::Error::new(io::ErrorKind::Other, "");
                        tx.send(Err(io_err)).await.ok(); // ignore error if rx closed
                        return Err(e)?;
                    }
                }
            }
            db_set(&path, STATE_FINISHED);
            // println!("m:got  {path}");
            anyhow::Ok(())
        })
        .await;
        if care!(result).is_err() {
            db_del(&path);
            // println!("m:err  {path}");
        }
    });
    MpscResponse::new(rx).into_response()
}

pub fn service() -> Router {
    db_init();
    Router::new()
        .route(
            "/mirror",
            MethodRouter::new().get(|| async { Html((include_src!("page.html") as [_; 1])[0]) }),
        )
        .route(
            "/mirror/*path",
            MethodRouter::new().get(|Path(p): Path<String>| async move {
                // http://mirror.nju.edu.cn/fedora/
                // http://mirrors.ustc.edu.cn/fedora/
                // http://mirrors.tuna.tsinghua.edu.cn/fedora/
                if let Some(r) = p.strip_prefix("fedora/") {
                    return handle(&p, r, "http://mirrors.ustc.edu.cn/fedora/").await;
                }
                if let Some(r) = p.strip_prefix("ubuntu/") {
                    return handle(&p, r, "http://mirrors.ustc.edu.cn/ubuntu/").await;
                }
                StatusCode::NOT_FOUND.into_response()
            }),
        )
}
