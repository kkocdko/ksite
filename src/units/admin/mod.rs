//! Admin console.

use crate::auth::auth_layer;
use crate::database::DB;
use crate::{include_src, log, strip_str};
use axum::body::Bytes;
use axum::extract::RawQuery;
use axum::middleware;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use std::time::{Duration, UNIX_EPOCH};

pub mod db {
    use super::*;
    pub async fn init() {
        DB.call(|db| {
            let sql = strip_str! {"
                CREATE TABLE IF NOT EXISTS admin (k BLOB PRIMARY KEY, v BLOB)
            "};
            let mut stmd = db.prepare(sql).unwrap();
            stmd.execute(()).unwrap();
        })
        .await
    }
    pub async fn set(k: String, v: Bytes) {
        DB.call(move |db| {
            let sql = strip_str! {"
                REPLACE INTO admin VALUES (?, ?)
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            stmd.execute((k.into_bytes(), v.as_ref())).unwrap();
        })
        .await
    }
    pub async fn get(k: String) -> Option<Bytes> {
        DB.call(move |db| {
            let sql = strip_str! {"
                SELECT v FROM admin WHERE k = ?
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            stmd.query_row((k.as_bytes(),), |r| {
                let v: Vec<u8> = r.get(0)?;
                Ok(Bytes::from(v))
            })
            .ok()
        })
        .await
    }
    pub async fn del(k: String) {
        DB.call(move |db| {
            let sql = strip_str! {"
                DELETE FROM admin WHERE k = ?
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            stmd.execute((k.as_bytes(),)).unwrap();
        })
        .await
    }
}

async fn post_handler(q: RawQuery, body: Bytes) -> Bytes {
    let q = q.0.unwrap();
    let k = q.as_str();
    log!("units::admin received op {k}");
    match k {
        "trigger_reset_auth_key" => {
            db::del("auth_key".to_owned()).await;
            // need restart to take effect
        }
        "trigger_restart_process" => {
            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(500));
                std::process::exit(0);
            });
        }
        "trigger_backup_database" => {
            crate::database::backup().await;
        }
        "get_recent_log" => {
            let mut file = crate::launcher::LOG_FILE.try_clone().unwrap();
            use std::io::{Read, Seek, SeekFrom};
            let max_len = 1024 * 128;
            let start_pos = file.metadata().unwrap().len().saturating_sub(max_len);
            file.seek(SeekFrom::Start(start_pos)).unwrap();
            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();
            return Bytes::from(buf);
        }
        "set_tls_ca" => {
            db::set("tls_ca".to_owned(), body).await;
        }
        "set_tls_cert" => {
            db::set("tls_cert".to_owned(), body).await;
        }
        "set_tls_key" => {
            db::set("tls_key".to_owned(), body).await;
        }
        "set_copilot_token" => {
            db::set("copilot_token".to_owned(), body).await;
        }
        "set_copilot_machineid" => {
            db::set("copilot_machineid".to_owned(), body).await;
        }
        "set_qqbot_device" => {
            db::set("qqbot_device".to_owned(), body).await;
        }
        "set_qqbot_token" => {
            db::set("qqbot_token".to_owned(), body).await;
        }
        "set_qqbot_notify_groups" => {
            db::set("qqbot_notify_groups".to_owned(), body).await;
        }
        "set_v2ex_cookies" => {
            db::set("v2ex_cookies".to_owned(), body).await;
        }
        _ => {
            log!(ERRO: "units::admin unknown op");
            return Bytes::from_static(b"unknown op");
        }
    }
    Bytes::from(format!(
        "finished, now = {}",
        UNIX_EPOCH.elapsed().unwrap().as_secs()
    ))
}

pub fn service() -> Router {
    crate::utils::block_on(async {
        db::init().await;
        if db::get("auth_key".to_owned()).await.is_none() {
            db::set("auth_key".to_owned(), Bytes::from(crate::auth::auth_key())).await;
        }
    });
    Router::new().route(
        "/admin",
        MethodRouter::new()
            .get(Html((include_src!("page.html") as [_; 1])[0]))
            .post(post_handler)
            .route_layer(middleware::from_fn(auth_layer)),
    )
}
