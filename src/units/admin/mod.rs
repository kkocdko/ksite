//! Admin console.

use crate::auth::auth_layer;
use crate::{include_src, log};
use axum::body::Bytes;
use axum::extract::RawQuery;
use axum::middleware;
use axum::response::Html;
use axum::routing::{MethodRouter, Router};
use std::time::{Duration, UNIX_EPOCH};

pub mod db {
    use crate::database::DB;
    use crate::strip_str;
    pub fn init() {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            CREATE TABLE IF NOT EXISTS admin (k BLOB PRIMARY KEY, v BLOB)
        "};
        let mut stmd = db.prepare(sql).unwrap();
        stmd.execute(()).unwrap();
    }
    pub fn set(k: &str, v: &[u8]) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            REPLACE INTO admin VALUES (?, ?)
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((k.as_bytes(), v)).unwrap();
    }
    pub fn get(k: &str) -> Option<Vec<u8>> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT v FROM admin WHERE k = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.query_row((k.as_bytes(),), |r| r.get(0)).ok()
    }
    pub fn del(k: &str) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            DELETE FROM admin WHERE k = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((k.as_bytes(),)).unwrap();
    }
}

async fn post_handler(q: RawQuery, body: Bytes) -> Bytes {
    let q = q.0.unwrap();
    let k = q.as_str();
    log!("units::admin received op {k}");
    match k {
        "trigger_reset_auth_key" => {
            db::del("auth_key");
            // need restart to take effect
        }
        "trigger_restart_process" => {
            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(500));
                std::process::exit(0);
            });
        }
        "trigger_backup_database" => {
            crate::database::backup();
        }
        "get_recent_log" => {
            let mut file = crate::launcher::LOG_FILE.try_clone().unwrap();
            use std::io::{Read, Seek, SeekFrom};
            let max_len = 16384;
            let start_pos = file.metadata().unwrap().len().saturating_sub(max_len);
            file.seek(SeekFrom::Start(start_pos)).unwrap();
            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();
            // file.take(max_len).read_to_string(&mut buf).unwrap();
            return Bytes::from(buf);
        }
        "set_ssl_cert" => {
            db::set("ssl_cert", &body);
        }
        "set_ssl_key" => {
            db::set("ssl_key", &body);
        }
        "set_copilot_token" => {
            db::set("copilot_token", &body);
        }
        "set_copilot_machineid" => {
            db::set("copilot_machineid", &body);
        }
        "set_qqbot_device" => {
            db::set("qqbot_device", &body);
        }
        "set_qqbot_token" => {
            db::set("qqbot_token", &body);
        }
        "set_qqbot_notify_groups" => {
            db::set("qqbot_notify_groups", &body);
        }
        "set_v2ex_cookies" => {
            db::set("v2ex_cookies", &body);
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
    db::init();
    if db::get("auth_key").is_none() {
        db::set("auth_key", crate::auth::auth_key().as_bytes());
    }
    Router::new().route(
        "/admin",
        MethodRouter::new()
            .get(Html((include_src!("page.html") as [_; 1])[0]))
            .post(post_handler)
            .route_layer(middleware::from_fn(auth_layer)),
    )
}
