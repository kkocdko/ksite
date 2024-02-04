//! WebDAV. The goal is fast and short, not to implement full RFC4918 + RFC2518.

use crate::database::DB;
use crate::utils::{escape_check_html, OptionResult};
use crate::{care, include_src, strip_str};
use axum::body::{Body, Bytes};
use axum::extract::{Path, Request};
use axum::handler::Handler;
use axum::http::{header::*, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{MethodRouter, Router};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod db {
    use super::*;
    pub const ENTRY_DIR: u64 = 0b_0000_0000_0000_0001;
    pub const ENTRY_READ_ONLY: u64 = 0b_0000_0000_0000_0010;
    pub const ENTRY_HREF: u64 = 0b_0000_0000_0000_1000;
    pub const ENTRY_GZIP: u64 = 0b_0000_0000_0001_0000;
    pub const ENTRY_STABLE: u64 = 0b_0000_0000_0100_0000;
    pub async fn init() {
        DB.call(|db| {
            // db.execute("DROP TABLE IF EXISTS dav_users", ()).unwrap();
            // db.execute("DROP TABLE IF EXISTS dav_entries", ()).unwrap();
            // dav_users: uid = "username", auth = "Basic dXNlcm5hbWU6cGFzc3dvcmQ="
            let sql = strip_str! {"
                CREATE TABLE IF NOT EXISTS dav_users (uid BLOB PRIMARY KEY, auth BLOB UNIQUE)
            "};
            let mut stmd = db.prepare(sql).unwrap();
            stmd.execute(()).unwrap();
            // dav_entries: eid = "username:/dir/file", data = "<bin data or empty>", time (modified, seconds) = 1706298055, size (bytes) = 4096, flag = 0b0000
            let sql = strip_str! {"
                CREATE TABLE IF NOT EXISTS dav_entries (eid BLOB PRIMARY KEY, data BLOB, time INTEGER, size INTEGER, flag INTEGER)
            "};
            let mut stmd = db.prepare(sql).unwrap();
            stmd.execute(()).unwrap();
        })
        .await
    }
    pub async fn get_user_uid(auth: String) -> Option<String> {
        DB.call(move |db| {
            let sql = strip_str! {"
                SELECT uid FROM dav_users WHERE auth = ?
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            stmd.query_row((auth.into_bytes(),), |r| {
                Ok(String::from_utf8(r.get(0)?).unwrap())
            })
            .ok()
        })
        .await
    }
    pub async fn set_user(uid: String, auth: String) {
        DB.call(move |db| {
            let sql = strip_str! {"
                REPLACE INTO dav_users VALUES (?, ?)
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            stmd.execute((uid.into_bytes(), auth.into_bytes())).unwrap();
        })
        .await
    }
    pub async fn set_entry(eid: String, data: Bytes, time: u64, size: u64, flag: u64) {
        DB.call(move |db| {
            let sql = strip_str! {"
                REPLACE INTO dav_entries VALUES (?, ?, ?, ?, ?)
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            stmd.execute((eid.into_bytes(), data.as_ref(), time, size, flag))
                .unwrap();
        })
        .await
    }
    pub async fn set_entry_flag(eid: String, flag: u64) {
        DB.call(move |db| {
            let sql = strip_str! {"
                UPDATE dav_entries SET flag = ? WHERE eid = ?
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            stmd.execute((flag, eid.as_bytes())).unwrap();
        })
        .await
    }
    pub async fn get_entry_data(eid: String) -> Option<Vec<u8>> {
        DB.call(move |db| {
            let sql = strip_str! {"
                SELECT data FROM dav_entries WHERE eid = ?
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            stmd.query_row((eid.as_bytes(),), |r| r.get(0)).ok()
        })
        .await
    }
    pub async fn get_entry_meta(eid: String) -> Option<(u64, u64, u64)> {
        DB.call(move |db| {
            let sql = strip_str! {"
                SELECT time, size, flag FROM dav_entries WHERE eid = ?
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            stmd.query_row((eid.as_bytes(),), |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .ok()
        })
        .await
    }
    pub async fn list_entry_meta(eid_prefix: String) -> Vec<(String, u64, u64, u64)> {
        DB.call(move |db| {
            let sql = strip_str! {"
                SELECT eid, time, size, flag FROM dav_entries WHERE eid LIKE ? AND eid NOT LIKE ?
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            // TODO: optimize
            let v_like = eid_prefix.to_owned() + "/%";
            let v_not_like = eid_prefix.to_owned() + "/%/%";
            stmd.query_map((v_like.into_bytes(), v_not_like.into_bytes()), |r| {
                Ok((
                    String::from_utf8(r.get(0)?).unwrap(),
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                ))
            })
            .unwrap()
            .map(|v| v.unwrap())
            .collect()
        })
        .await
    }
    pub async fn del_entry_recursive(eid: String) {
        DB.call(move |db| {
            let sql = strip_str! {"
                DELETE FROM dav_entries WHERE eid = ? OR eid LIKE ?
            "};
            let mut stmd = db.prepare_cached(sql).unwrap();
            let v_like = eid.to_owned() + "/%";
            stmd.execute((eid.into_bytes(), v_like.into_bytes()))
                .unwrap();
        })
        .await
    }
}

async fn dav_handler(prefix: &'static str, mut req: Request) -> anyhow::Result<Response> {
    const MAX_SIZE: usize = 1024 * 1024 * 16;
    let method = req.method().as_str();
    if method == "OPTIONS" {
        return Ok(([
            ("Allow", "OPTIONS, DELETE, PROPPATCH, COPY, MOVE, PROPFIND"),
            ("Dav", "1, 2"),
        ])
        .into_response());
    }
    let Some(auth) = req.headers().get(AUTHORIZATION) else {
        return Ok((StatusCode::UNAUTHORIZED, [(WWW_AUTHENTICATE, "Basic")]).into_response());
    };
    let uid = db::get_user_uid(auth.to_str()?.to_owned()).await.e()?;
    let pathname = req.uri().path().trim_start_matches(prefix);
    let eid = uid.to_owned() + ":" + pathname.trim_end_matches('/');
    match method {
        "PUT" | "MKCOL" => {
            if let Some((_, _, flag)) = db::get_entry_meta(eid.to_owned()).await {
                if flag & db::ENTRY_READ_ONLY != 0 {
                    return Err(anyhow::anyhow!("read only"));
                }
            }
            let (parent, _cur_name) = eid.rsplit_once('/').e()?;
            let (_, _, flag) = db::get_entry_meta(parent.to_owned()).await.e()?;
            if flag & db::ENTRY_READ_ONLY != 0 {
                return Err(anyhow::anyhow!("read only"));
            }
            if flag & db::ENTRY_DIR == 0 {
                return Err(anyhow::anyhow!("parent is not dir"));
            }
            let time = UNIX_EPOCH.elapsed().unwrap().as_secs();
            match method {
                "PUT" => {
                    let data = axum::body::to_bytes(req.into_body(), MAX_SIZE).await?;
                    let size = data.len() as _;
                    db::set_entry(eid, data, time, size, 0).await;
                }
                "MKCOL" => {
                    db::set_entry(eid, Bytes::new(), time, 0, db::ENTRY_DIR).await;
                }
                _ => unreachable!(),
            }
            Ok(StatusCode::CREATED.into_response())
        }
        "DELETE" => {
            let (_, _, flag) = db::get_entry_meta(eid.to_owned()).await.e()?;
            if flag & db::ENTRY_READ_ONLY != 0 {
                return Err(anyhow::anyhow!("read only"));
            }
            db::del_entry_recursive(eid.to_owned()).await; // TODO
            Ok(StatusCode::OK.into_response())
        }
        "COPY" | "MOVE" => {
            let (time, size, flag) = db::get_entry_meta(eid.to_owned()).await.e()?;
            if flag & db::ENTRY_READ_ONLY != 0 {
                return Err(anyhow::anyhow!("read only"));
            }
            #[allow(clippy::unnecessary_to_owned)] // false positive
            let dest = Uri::from_maybe_shared(req.headers().get("Destination").e()?.to_owned())?;
            let dest = dest.path().trim_start_matches(prefix);
            let dest_eid = uid + ":" + dest.trim_end_matches('/');
            if flag & db::ENTRY_DIR == 0 {
                let data = db::get_entry_data(eid.to_owned()).await.unwrap();
                db::set_entry(dest_eid, Bytes::from(data), time, size, flag).await;
                if method == "MOVE" {
                    // TODO: opti
                    db::del_entry_recursive(eid).await;
                }
            } else {
                // TODO
                return Err(anyhow::anyhow!("is dir, todo"));
            }
            Ok(StatusCode::OK.into_response())
        }
        "GET" | "HEAD" => {
            let (time, size, flag) = db::get_entry_meta(eid.to_owned()).await.e()?;
            if flag & db::ENTRY_DIR != 0 {
                return Err(anyhow::anyhow!("is dir"));
            }
            if flag & db::ENTRY_HREF != 0 {
                let v = HeaderValue::try_from(db::get_entry_data(eid).await.e()?)?;
                return Ok((StatusCode::TEMPORARY_REDIRECT, [(LOCATION, v)]).into_response());
            }
            if let Some(v) = req.headers().get(IF_MODIFIED_SINCE) {
                let t = httpdate::parse_http_date(v.to_str()?)?;
                let t = t.duration_since(UNIX_EPOCH)?.as_secs();
                if t >= time {
                    return Ok(StatusCode::NOT_MODIFIED.into_response());
                }
            }
            let mut res = match method {
                "GET" => axum::body::Body::from(db::get_entry_data(eid).await.e()?),
                "HEAD" => axum::body::Body::empty(),
                _ => unreachable!(),
            }
            .into_response();
            let stamp = httpdate::fmt_http_date(UNIX_EPOCH + Duration::from_secs(time));
            res.headers_mut().insert(LAST_MODIFIED, stamp.try_into()?);
            res.headers_mut().insert(CONTENT_LENGTH, size.into());
            if flag & db::ENTRY_GZIP != 0 {
                let v = HeaderValue::from_static("gzip");
                res.headers_mut().insert(CONTENT_ENCODING, v);
            }
            if flag & db::ENTRY_STABLE != 0 {
                let v = HeaderValue::from_static("max-age=600,stale-while-revalidate=31536000");
                res.headers_mut().insert(CACHE_CONTROL, v);
            }
            Ok(res)
        }
        "PROPFIND" => {
            // the "depth" is ignored here
            let mut body = String::new();
            body += r#"<?xml version="1.0" encoding="utf-8" ?><D:multistatus xmlns:D="DAV:">"#;
            let (time, size, flag) = db::get_entry_meta(eid.to_owned()).await.e()?;
            let mut entries = Vec::new();
            entries.push((eid.to_owned(), time, size, flag));
            if flag & db::ENTRY_DIR == 0 {
            } else {
                entries.extend(db::list_entry_meta(eid.to_owned()).await);
            }
            for (eid, time, size, flag) in entries {
                let (uid, pathname) = eid.split_once(':').e()?;
                if flag & db::ENTRY_DIR == 0 {
                    body += "<D:response><D:href>";
                    body += prefix;
                    body += pathname;
                    body += "</D:href><D:propstat><D:prop><D:displayname>";
                    body += pathname.rsplit_once('/').e()?.1;
                    body += "</D:displayname><D:getcontentlength>";
                    body += &size.to_string();
                    body += "</D:getcontentlength><D:getlastmodified>";
                    body += &httpdate::fmt_http_date(UNIX_EPOCH + Duration::from_secs(time));
                    body += r#"</D:getlastmodified><D:getcontenttype></D:getcontenttype><D:resourcetype></D:resourcetype></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>"#;
                } else {
                    body += "<D:response><D:href>";
                    body += prefix;
                    body += pathname;
                    body += "/";
                    body += "</D:href><D:propstat><D:prop><D:displayname>";
                    body += pathname
                        .rsplit_once('/')
                        .map(|(_parent, v)| v)
                        .unwrap_or_default();
                    body += "</D:displayname><D:getlastmodified>";
                    body += &httpdate::fmt_http_date(UNIX_EPOCH + Duration::from_secs(time));
                    body += r#"</D:getlastmodified><D:resourcetype><D:collection/></D:resourcetype></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>"#;
                }
            }
            body += r#"</D:multistatus>"#;
            Ok((
                StatusCode::MULTI_STATUS,
                [(CONTENT_TYPE, "text/xml; charset=utf-8")],
                body,
            )
                .into_response())
        }
        _ => Err(anyhow::anyhow!("unimplemented method")),
    }
}

async fn api_handler(mut req: Request) -> anyhow::Result<Response> {
    let mut get_field = |k| {
        let v = req.headers_mut().remove(k);
        v.and_then(|v| Some(v.to_str().ok()?.to_owned())).e()
    };
    Ok(match get_field("op_")?.as_str() {
        "signup" => {
            let uid = get_field("uid_")?;
            if !uid
                .as_bytes()
                .iter()
                .all(|&c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
            {
                return Err(anyhow::anyhow!("uid contains invalid chars"));
            }
            let auth = get_field("auth_")?;
            db::set_user(uid.to_owned(), auth.to_owned()).await; // TODO
            let time = UNIX_EPOCH.elapsed().unwrap().as_secs();
            db::set_entry(uid.to_owned() + ":", Bytes::new(), time, 0, db::ENTRY_DIR).await;
            StatusCode::OK.into_response()
        }
        "trigger_flag_recursive" => {
            let eid = get_field("eid_")?;
            let not = get_field("not_").is_ok();
            let apply_dir = get_field("apply_dir_").is_ok(); // apply flag on dir, or only non-dir
            let trigger_flag: u64 = get_field("flag_")?.parse()?;
            let mut list = Vec::new(); // certainly that std VecDeque will be better, however it's more complex inside
            list.push(eid);
            let mut idx = 0;
            while idx != list.len() {
                let eid = std::mem::take(&mut list[idx]);
                let Some((_, _, flag)) = db::get_entry_meta(eid.to_owned()).await else {
                    continue;
                };
                if flag & db::ENTRY_DIR == 0 || apply_dir {
                    let new_flag = match not {
                        true => flag & !trigger_flag,
                        false => flag | trigger_flag,
                    };
                    db::set_entry_flag(eid.to_owned(), new_flag).await;
                }
                if flag & db::ENTRY_DIR != 0 {
                    for (eid, _, _, _) in db::list_entry_meta(eid).await {
                        list.push(eid);
                    }
                }
                idx += 1;
            }
            StatusCode::OK.into_response()
        }
        _ => return Err(anyhow::anyhow!("unknown op")),
    })
}

pub fn service() -> Router {
    tokio::spawn(db::init());
    const DAV_PATH_PREFIX: &str = "/dav";
    let api_router = axum::routing::any(|req: Request| async {
        match care!(api_handler(req).await) {
            Ok(res) => res,
            Err(_) => StatusCode::BAD_REQUEST.into_response(),
        }
    })
    .layer(axum::middleware::from_fn(crate::auth::auth_layer));
    let any_router = axum::routing::any(|req: Request| async {
        if req.uri().path() == DAV_PATH_PREFIX && req.method() == "GET" {
            let mut res = Html((include_src!("page.html") as [_; 1])[0]).into_response();
            let cache_control_value =
                HeaderValue::from_static("max-age=600,stale-while-revalidate=31536000");
            res.headers_mut().insert(CACHE_CONTROL, cache_control_value);
            res
        } else if req.uri().path() == DAV_PATH_PREFIX && req.method() == "POST" {
            api_router.call(req, ()).await
        } else if let Ok(res) = care!(dav_handler(DAV_PATH_PREFIX, req).await) {
            res
        } else {
            StatusCode::BAD_REQUEST.into_response()
        }
    });
    Router::new()
        .route("/dav", any_router.clone())
        .route("/dav/", any_router.clone())
        .route("/dav/*path", any_router)
}
