//! WebDAV.

use crate::auth::auth_layer;
use crate::utils::{escape_check_html, OptionResult};
use crate::{care, include_src};
use axum::body::{Body, Bytes};
use axum::extract::{Path, Request};
use axum::http::{header::*, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{MethodRouter, Router};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod db {
    use crate::database::DB;
    use crate::strip_str;
    pub const ENTRY_DIR: u64 = 0b_0000_0000_0000_0001;
    pub const ENTRY_GZIP: u64 = 0b_0000_0000_0001_0000;
    pub fn init() {
        let db = DB.lock().unwrap();
        // {
        //     // TODO
        //     db.execute("DROP TABLE IF EXISTS dav_users", ()).unwrap();
        //     db.execute("DROP TABLE IF EXISTS dav_entries", ()).unwrap();
        // }
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
    }
    pub fn get_user_uid(auth: &str) -> Option<String> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT uid FROM dav_users WHERE auth = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.query_row((auth.as_bytes(),), |r| {
            Ok(String::from_utf8(r.get(0)?).unwrap())
        })
        .ok()
    }
    pub fn set_user(uid: &str, auth: &str) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            REPLACE INTO dav_users VALUES (?, ?)
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((uid.as_bytes(), auth.as_bytes())).unwrap();
    }
    pub fn set_entry(eid: &str, data: &[u8], time: u64, size: u64, flag: u64) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            REPLACE INTO dav_entries VALUES (?, ?, ?, ?, ?)
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((eid.as_bytes(), data, time, size, flag))
            .unwrap();
    }
    pub fn get_entry_data(eid: &str) -> Option<Vec<u8>> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT data FROM dav_entries WHERE eid = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.query_row((eid.as_bytes(),), |r| r.get(0)).ok()
    }
    pub fn get_entry_meta(eid: &str) -> Option<(u64, u64, u64)> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT time, size, flag FROM dav_entries WHERE eid = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.query_row((eid.as_bytes(),), |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .ok()
    }
    pub fn list_entry_meta(eid_prefix: &str) -> Vec<(String, u64, u64, u64)> {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            SELECT eid, time, size, flag FROM dav_entries WHERE eid LIKE ? AND eid NOT LIKE ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        // TODO: optimize
        let v_like = eid_prefix.to_owned() + "/%";
        let v_not_like = eid_prefix.to_owned() + "/%/%";
        stmd.query_map((v_like.as_bytes(), v_not_like.as_bytes()), |r| {
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
    }
    pub fn del_entry(eid: &str) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            DELETE FROM dav_entries WHERE eid = ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        stmd.execute((eid.as_bytes(),)).unwrap();
    }
    pub fn del_entry_dir(eid: &str) {
        let db = DB.lock().unwrap();
        let sql = strip_str! {"
            DELETE FROM dav_entries WHERE eid = ? OR eid LIKE ?
        "};
        let mut stmd = db.prepare_cached(sql).unwrap();
        let v_like = eid.to_owned() + "/%";
        stmd.execute((eid.as_bytes(), v_like.as_bytes())).unwrap();
    }
}

/*
https://datatracker.ietf.org/doc/html/rfc4918
https://datatracker.ietf.org/doc/html/rfc2518
https://github.com/sigoden/dufs/blob/main/src/server.rs#L329
https://en.wikipedia.org/wiki/WebDAV

curl http://username:password@127.0.0.1:9630
[kkocdko@klf apps]$ ./busybox nc -p 9630 -l
GET / HTTP/1.1
Host: 127.0.0.1:9630
Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=
User-Agent: curl/8.2.1
Accept: *

curl -vvvk -X POST --data-raw '{"op":"create_user","uid":"username","auth":"Basic dXNlcm5hbWU6cGFzc3dvcmQ="}' http://127.0.0.1:9304/dav

curl -vvvk -X PUT --data-raw 'hello' http://username:password@127.0.0.1:9304/dav/a

curl -vvvk http://username:password@127.0.0.1:9304/dav/a?a

*/

async fn dav_handler(prefix: &'static str, mut req: Request) -> anyhow::Result<Response> {
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
    let uid = db::get_user_uid(auth.to_str().unwrap()).e()?;
    let pathname = req.uri().path().trim_start_matches(prefix);
    let eid = uid.to_owned() + ":" + pathname.trim_end_matches("/");
    dbg!(&eid);
    match method {
        "PUT" | "MKCOL" => {
            let (parent, _cur_name) = eid.rsplit_once('/').e()?;
            let (_, _, flag) = db::get_entry_meta(&parent).e()?;
            if flag & db::ENTRY_DIR == 0 {
                return Err(anyhow::anyhow!("parent is not dir"));
            }
            let time = UNIX_EPOCH.elapsed().unwrap().as_secs();
            match method {
                "PUT" => {
                    let mut flag = 0;
                    if matches!(req.headers().get(CONTENT_ENCODING), Some(v) if v == "gzip") {
                        flag |= db::ENTRY_GZIP;
                    }
                    let data = axum::body::to_bytes(req.into_body(), 1024 * 1024).await?;
                    db::set_entry(&eid, &data, time, data.len() as _, flag);
                }
                "MKCOL" => {
                    db::set_entry(&eid, &[], time, 0, db::ENTRY_DIR);
                }
                _ => unreachable!(),
            }
            Ok(StatusCode::OK.into_response())
        }
        "DELETE" => {
            let (time, size, flag) = db::get_entry_meta(&eid).e()?;
            if flag & db::ENTRY_DIR == 0 {
                db::del_entry(&eid);
            } else {
                // TODO
                db::del_entry_dir(&eid);
            }
            Ok(StatusCode::OK.into_response())
        }
        "COPY" | "MOVE" => {
            let (time, size, flag) = db::get_entry_meta(&eid).e()?;
            let dest = Uri::from_maybe_shared(req.headers().get("Destination").e()?.to_owned())?;
            let dest = dest.path().trim_start_matches(prefix);
            let dest_eid = uid + ":" + dest.trim_end_matches("/");
            if flag & db::ENTRY_DIR == 0 {
                let data = db::get_entry_data(&eid).unwrap();
                db::set_entry(&dest_eid, &data, time, size, flag);
                if method == "MOVE" {
                    // TODO: opti
                    db::del_entry(&eid);
                }
            } else {
                // TODO
                return Err(anyhow::anyhow!("is dir, todo"));
            }
            Ok(StatusCode::OK.into_response())
        }
        "GET" | "HEAD" => {
            let (time, size, flag) = db::get_entry_meta(&eid).e()?;
            if flag & db::ENTRY_DIR != 0 {
                return Err(anyhow::anyhow!("is dir"));
            }
            let mut res = match method {
                "GET" => axum::body::Body::from(db::get_entry_data(&eid).unwrap()),
                "HEAD" => axum::body::Body::empty(),
                _ => unreachable!(),
            }
            .into_response();
            let header = res.headers_mut();
            let stamp = httpdate::fmt_http_date(UNIX_EPOCH + Duration::from_secs(time));
            header.insert(LAST_MODIFIED, stamp.try_into().unwrap());
            header.insert(CONTENT_LENGTH, size.to_string().try_into().unwrap()); // TODO: range
            if flag & db::ENTRY_GZIP != 0 && method == "GET" {
                header.insert(CONTENT_ENCODING, HeaderValue::from_static("gzip"));
            }
            Ok(res)
        }
        "PROPFIND" => {
            // the "depth" is ignored here
            let mut body = String::new();
            body += r#"<?xml version="1.0" encoding="utf-8" ?><D:multistatus xmlns:D="DAV:">"#;
            let (time, size, flag) = db::get_entry_meta(&eid).e()?;
            let mut entries = Vec::new();
            entries.push((eid.to_owned(), time, size, flag));
            if flag & db::ENTRY_DIR == 0 {
            } else {
                entries.extend(db::list_entry_meta(&eid));
            }
            for (eid, time, size, flag) in entries {
                let (uid, pathname) = eid.split_once(':').e()?;
                if flag & db::ENTRY_DIR == 0 {
                    body += "<D:response><D:href>";
                    body += pathname;
                    body += "</D:href><D:propstat><D:prop><D:displayname>";
                    let Some((_parent, cur_name)) = pathname.rsplit_once('/') else {
                        return Err(anyhow::anyhow!("invalid pathname"));
                    };
                    body += cur_name;
                    body += "</D:displayname><D:getcontentlength>";
                    body += &size.to_string();
                    body += "</D:getcontentlength><D:getlastmodified>";
                    body += &httpdate::fmt_http_date(UNIX_EPOCH + Duration::from_secs(time));
                    body += r#"</D:getlastmodified><D:getcontenttype></D:getcontenttype><D:resourcetype></D:resourcetype></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>"#;
                } else {
                    body += "<D:response><D:href>";
                    body += pathname;
                    body += "/";
                    body += "</D:href><D:propstat><D:prop><D:displayname>";
                    let display_name = pathname
                        .rsplit_once('/')
                        .map(|(_parent, v)| v)
                        .unwrap_or_default();
                    body += display_name;
                    body += "</D:displayname><D:getlastmodified>";
                    body += &httpdate::fmt_http_date(UNIX_EPOCH + Duration::from_secs(time));
                    body += r#"</D:getlastmodified><D:resourcetype><D:collection/></D:resourcetype></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>"#;
                }
            }
            // <D:response><D:href>/dav/local/proxy/</D:href><D:propstat><D:prop><D:displayname>proxy</D:displayname><D:supportedlock><D:lockentry xmlns:D="DAV:"><D:lockscope><D:exclusive/></D:lockscope><D:locktype><D:write/></D:locktype></D:lockentry></D:supportedlock><D:getlastmodified>Mon, 22 Jan 2024 13:30:59 GMT</D:getlastmodified><D:creationdate>Mon, 22 Jan 2024 13:30:59 GMT</D:creationdate><D:resourcetype><D:collection xmlns:D="DAV:"/></D:resourcetype></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>
            // <D:response><D:href>/dav/local/proxy/config.jsonc</D:href><D:propstat><D:prop><D:displayname>config.jsonc</D:displayname><D:getcontentlength>39765</D:getcontentlength><D:getlastmodified>Fri, 26 Jan 2024 17:49:04 GMT</D:getlastmodified><D:creationdate>Fri, 26 Jan 2024 17:49:04 GMT</D:creationdate><D:getcontenttype></D:getcontenttype><D:getetag>"17adf6f011f3c2be9b55"</D:getetag><D:resourcetype></D:resourcetype></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>
            body += r#"</D:multistatus>"#;
            dbg!(&body);
            Ok((
                StatusCode::MULTI_STATUS,
                [(CONTENT_TYPE, "application/xml; charset=utf-8")],
                body,
            )
                .into_response())
        }
        _ => Err(anyhow::anyhow!("unknown method")),
    }
}

// gio mount dav://127.0.0.1:9304/dav

async fn api_handler(body: String) -> anyhow::Result<Response> {
    let body = serde_json::from_slice::<serde_json::Value>(body.as_bytes())?;
    let get_field = |k| body.get(k).and_then(|v| v.as_str()).e();
    Ok(match get_field("op")? {
        "create_user" => {
            let uid = get_field("uid")?;
            if !uid
                .as_bytes()
                .iter()
                .all(|&c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
            {
                return Err(anyhow::anyhow!("uid contains invalid chars"));
            }
            let auth = get_field("auth")?;
            db::set_user(uid, auth); // TODO
            let time = UNIX_EPOCH.elapsed().unwrap().as_secs();
            db::set_entry(&(uid.to_owned() + ":"), &[], time, 0, db::ENTRY_DIR);
            StatusCode::OK.into_response()
        }
        _ => return Err(anyhow::anyhow!("unknown op")),
    })
}

pub fn service() -> Router {
    db::init();
    const DAV_PATH_PREFIX: &str = "/dav";
    let any_router = axum::routing::any(|req: Request| async {
        if req.uri().path() == DAV_PATH_PREFIX && matches!(req.method().as_str(), "GET" | "POST") {
            let body =
                match <String as axum::extract::FromRequest<()>>::from_request(req, &()).await {
                    Ok(v) => v,
                    Err(e) => return e.into_response(),
                };
            if let Ok(res) = care!(api_handler(body).await) {
                return res;
            }
        } else {
            if let Ok(res) = care!(dav_handler(DAV_PATH_PREFIX, req).await) {
                return res;
            }
        }
        StatusCode::BAD_REQUEST.into_response()
    });
    Router::new()
        .route("/dav", any_router.clone())
        .route("/dav/", any_router.clone())
        .route("/dav/*path", any_router)
        .layer(axum::middleware::from_fn(
            |req: Request<Body>, next: axum::middleware::Next| async {
                println!("================");
                println!("> req > {} {} {:?}", req.method(), req.uri(), req.headers());
                println!("================");
                next.run(req).await
            },
        ))
}
