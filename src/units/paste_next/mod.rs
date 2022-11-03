/*
剪贴板 & 轻量文件存储

features:
* 账户登录
* 强加密
* 文件分享
* 轻量，快速，不使用框架的界面

details:
* fid 经过优化的自增 （保留前 32 个 ID？）
* fid 不要超过 i64（测试 u64 的可能性？）
* fid 在前端从 str 转成 int
* fid 生成完美哈希
* 写入原始内容，前端预览时自行处理转义
* 不要做服务端解密，前端做边解密边下载
* 默认使用用户密码加密，可自定义加密。用 mime 来存储是否自定义加密的信息

evolution:
* 防止基于时间的侧信道攻击
* 防止重放攻击
* 修改用户名，不对称算力验证防攻击
* 邮箱关联
* 类似 git fork, 但不使用 cow
* 大文件增量同步
* 页面缓存
* 会员制

sessions?
全部 public，未分享的用用户密码加密?
读写本地文件账户登录，加密
原生js糊的轻量的界面，借鉴一点点 react 之类的东西
在用户之间分享，转移所有权
尽量优化性能
内部用数字存储 fid，文件名和路径也是数字？
账户创建 file 的速度限制。
区分创建与插入？评估性能影响

protobuf?
webdav?

https://github.com/lettre/lettre
https://www.runoob.com/sqlite/sqlite-intro.html
https://github.com/su18/wooyun-drops/blob/b2a5416/papers/%E5%8A%A0%E7%9B%90hash%E4%BF%9D%E5%AD%98%E5%AF%86%E7%A0%81%E7%9A%84%E6%AD%A3%E7%A1%AE%E6%96%B9%E5%BC%8F.md
https://docs.rs/rustls/latest/rustls/internal/msgs/enums/enum.HashAlgorithm.html#variant.SHA256
https://docs.rs/ring/latest/ring/digest/fn.digest.html
file:///home/kkocdko/misc/Markdown_1666085837234.html

密码hash加盐

/paste/raw/:id

-----
/ksite
/ksite.db

* ===== SIGNUP
client: post(id, h = hash(id + password))
server: store(id, salt = random(), hash(salt + h)), ret(result)

* ===== LOGIN
client: post(id, h = hash(id + password))
server: compare(hash(salt + h), secret), ret(token = hash(seed + id))

* ===== OPERATION
client: post(operation, token)
server: compare(token, target = hash(time + id)), ret(result)

*/
use crate::ticker::Ticker;
use crate::{db, include_page};
use axum::body::{Body, HttpBody as _};
use axum::extract::FromRequest;
use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum::response::{Html, IntoResponse, Json, Response};
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use std::future::Future;
use std::io::Write as _;
use std::mem::{swap, MaybeUninit};
use std::pin::Pin;
// use tokio_rustls::rustls::internal::msgs::codec::Codec as _;
// use tokio_rustls::rustls::internal::msgs::enums::HashAlgorithm;
// use bytes::{BufMut, BytesMut};
// pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn db_init() {
    // uid: user id
    // upw: user secret
    // mail: user email address
    // level: user level (admin / vip / banned / normal)
    // fid: file id (u64, != 0)
    // desc: file description
    // mime: file mime, use this as the content-type, and is encrypted flag
    db! {"
        CREATE TABLE IF NOT EXISTS paste_user
        (uid BLOB PRIMARY KEY, upw BLOB, mail BLOB, level INTEGER)
    "}
    .unwrap();
    db! {"
        CREATE TABLE IF NOT EXISTS paste_data
        (fid INTEGER PRIMARY KEY AUTOINCREMENT, size INTEGER, uid BLOB, desc BLOB, mime BLOB)
    "}
    .unwrap();
    // TODO: insert one to avoid 0th file?
}
fn db_user_cu(uid: &[u8], upw: &[u8], mail: &[u8], level: u8) {
    db! {"
        REPLACE INTO paste_user
        VALUES (?1, ?2, ?3, ?4)
    ",[uid, upw, mail, level as i64]}
    .unwrap();
}
fn db_user_r(uid: &[u8]) -> Option<(Vec<u8>, Vec<u8>, u8)> {
    db! {"
        SELECT * FROM paste_user
        WHERE uid = ?
    ", [uid], ^(1, 2, 3)}
    .ok() // TODO: convert i64 to u64?
}
// fn db_user_d(uid: &[u8]) {
//     db! {"
//         DELETE FROM paste_user
//         WHERE uid = ?
//     ", [uid]}
//     .unwrap();
// }
fn db_data_c(size: u64, uid: &[u8], desc: &[u8], mime: &[u8]) -> u64 {
    db! {"
        INSERT INTO paste_data
        VALUES (NULL, ?1, ?2, ?3, ?4)
    ", [size, uid, desc, mime], &}
    .unwrap() as _
}
// fn db_data_u_desc(fid: i64, desc: &[u8]) -> bool {
//     db! {"
//         UPDATE paste_data
//         SET desc = ?2
//         WHERE fid = ?1
//     ", [fid, desc]}
//     .is_ok()
// }
// fn db_data_u_mime(fid: i64, mime: &[u8]) -> bool {
//     db! {"
//         UPDATE paste_data
//         SET mime = ?2
//         WHERE fid = ?1
//     ", [fid, mime]}
//     .is_ok()
// }
// fn db_data_r(fid: i64) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>)> {
//     db!("SELECT * FROM paste_data WHERE fid = ?", [fid], ^(1, 2, 3)).ok()
// }
fn db_data_r_by_user(uid: &[u8]) -> Vec<(u64, u64, Vec<u8>, Vec<u8>)> {
    db!(
        "SELECT * FROM paste_data WHERE uid = ?",
        [uid],
        (0, 1, 3, 4)
    )
    .unwrap()
}
fn db_data_d(fid: i64) -> bool {
    db!("DELETE FROM paste_data WHERE fid = ?", [fid]).is_ok()
}

// /// Convert fid integer to string.
// fn fid_i2s(i: i64, buf: &mut [u8; FID_MAX_LEN]) -> &[u8] {
//     const L: usize = FID_CHARS.len();
//     let mut i = i as usize;
//     let mut p = FID_MAX_LEN - 1;
//     while i != 0 {
//         unsafe { *buf.get_unchecked_mut(p) = *FID_CHARS.get_unchecked(i % L) };
//         p -= 1;
//         i /= L;
//     }
//     &buf[p + 1..]
// }

// /// Convert fid string to integer.
// fn fid_s2i(s: &[u8]) -> i64 {
//     const L: i64 = FID_CHARS.len() as _;
//     let mut ret = 0;
//     for c in s {
//         let c = match c {
//             b'0'..=b'9' => c - b'0',
//             b'a'..=b'z' => c - b'a' + 10,
//             _ => unreachable!(),
//         } as i64;
//         ret = ret * L + c;
//     }
//     ret
// }

// const FID_HASH_TABLE: u64 = 0x922d8336cc9cad34; // random magic number

/// Convert fid, hashed -> raw
fn fid_h2r(i: u64) -> u64 {
    i
}

/// Convert fid, raw -> hashed
fn fid_r2h(i: u64) -> u64 {
    i
}

/// Promised that every comparison cost the same time.
fn slow_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false; // this is safe because everyone knows the hash len
    }
    let mut diff = 0;
    for i in 0..b.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// Token Module
///
/// The token was regenerated every 30 minutes, and expired after 1 hour. There are 3 seed in the
/// pool, request renew on client should response the next token and the valid timestamp.
///
/// ```norust
/// timestamp: u32 = minutes, level: u8, uid: &[u8]
/// token: &[u8] =
///     hash(seed[timestamp] + level:u8 + uid)
///     + timestamp:u32_le_bytes
///     + level:u8
///     + b'.'
///     + uid:&[u8]
///
/// | 30 min |
/// | --- seed 00 --- |
///          | --- seed 01 --- |
///                   | --- seed 02 --- |
///                            | --- seed 00 --- |
/// ```
mod token {
    use ring::digest::Digest;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::UNIX_EPOCH;

    static mut SEED_POOL: [[u8; 32]; 3] = [[0; 32]; 3];
    static CURRENT_TIMESTAMP: AtomicU32 = AtomicU32::new(0);

    fn hash(seed_idx: usize, level: u8, uid: &[u8]) -> Digest {
        let seed = unsafe { SEED_POOL[seed_idx] };
        let mut buf = Vec::new(); // TODO: avoid dynamic memory
        buf.extend(seed);
        buf.push(level);
        buf.extend(uid);
        ring::digest::digest(&ring::digest::SHA256, &buf) // it's hardware accelerated!
    }

    /// Get the seed index (`0`, `1` or `2`) by timestamp, returns `usize::MAX` if the timestamp
    /// was outdated (after 1 hour).
    fn timestamp2idx(timestamp: u32) -> usize {
        let current_timestamp = CURRENT_TIMESTAMP.load(Ordering::Relaxed);
        const LIMIT: u32 = 60 - 5; // WARNING: avoid edge case?
        if current_timestamp.saturating_sub(timestamp) > LIMIT {
            return usize::MAX; // outdated
        }
        (timestamp % 90 / 30) as _
    }

    /// Generate token in current time.
    pub fn current(uid: &[u8], level: u8) -> String {
        let timestamp = CURRENT_TIMESTAMP.load(Ordering::Relaxed);
        let seed_idx = timestamp2idx(timestamp);
        let mut buf = Vec::new();
        buf.extend(hash(seed_idx, level, uid).as_ref());
        buf.extend(timestamp.to_le_bytes());
        buf.extend(level.to_le_bytes());
        buf = super::bytes2hex(&buf);
        buf.push(b'.');
        buf.extend(uid);
        String::from_utf8(buf).unwrap()
    }

    /// Extract (uid, level).
    pub fn vertify(token: &[u8]) -> Result<(&[u8], u8), ()> {
        if token.len() < 76 {
            return Err(()); // too short
        }
        let sha256: [u8; 32] = super::hex2bytes(&token[..64])?;
        let timestamp = u32::from_le_bytes(super::hex2bytes(&token[64..72])?);
        let level = super::hex2bytes::<1>(&token[72..74])?[0];
        let uid = &token[75..];
        let seed_idx = timestamp2idx(timestamp);
        match hash(seed_idx, level, &uid).as_ref() == sha256 {
            true => Ok((uid, level)),
            false => Err(()),
        }
        // In export interface, we use (uid, level); in this module, due to the flexibility, we use
        // the token format that uid is behind the level.
    }

    pub fn renew_tick() {
        // WARNING: edge case?
        let current_timestamp = (UNIX_EPOCH.elapsed().unwrap().as_secs() / 60) as u32;
        let seed_idx = timestamp2idx(current_timestamp);
        unsafe {
            if seed_idx == usize::MAX {
                // is init
                for seed in &mut SEED_POOL {
                    for v in seed {
                        *v = rand::random();
                    }
                }
            } else {
                let next_idx = (seed_idx + 1) % 3;
                for v in &mut SEED_POOL[next_idx] {
                    *v = rand::random();
                }
                // if this function runs overfrquency, the next seed will renew many times, but
                // it's allowed and sound.
            }
        }
        // store **AFTER** the seed pool was renewed
        CURRENT_TIMESTAMP.store(current_timestamp, Ordering::SeqCst);
    }
}

/// Operations from client.
enum Op<'a> {
    /// Create a new user.
    Signup {
        uid: &'a [u8],
        upw: &'a [u8],
        mail: &'a [u8],
    },
    /// User login or token renew.
    Login { uid: &'a [u8], upw: &'a [u8] },
    /// Change user profile.
    // Profile {
    //     upw: &'a [u8],
    //     mail: &'a [u8],
    //     level: &'a [u8],
    //     token: &'a [u8],
    // },
    /// Get the files list.
    List { token: &'a [u8] },
    /// Create a file.
    Create {
        token: &'a [u8],
        desc: &'a [u8],
        mime: &'a [u8],
        body: Body,
    },
    // Update {},
    // Delete {},
    // Get {
    //     fid: i64,
    // },
    // Info {
    //     fid: i64,
    //     token: &'a [u8],
    // },
}

struct IntoOp(Request<Body>);

impl IntoOp {
    fn into_op(&mut self) -> Result<Op, ()> {
        let mut body = Body::empty(); // TODO: optimize unnecessary body extact
        swap(self.0.body_mut(), &mut body);
        let headers = self.0.headers();
        macro_rules! v {
            ($k:expr) => {{
                match headers.get($k) {
                    Some(v) => v.as_bytes(),
                    None => return Err(()),
                }
            }};
        }
        Ok(match v!("$op") {
            b"signup" => Op::Signup {
                uid: v!("$uid"),
                upw: v!("$upw"),
                mail: v!("$mail"),
            },
            b"login" => Op::Login {
                uid: v!("$uid"),
                upw: v!("$upw"),
            },
            b"create" => Op::Create {
                token: v!("$token"),
                desc: v!("$desc"),
                mime: v!("$mime"),
                body,
            },
            b"list" => Op::List {
                token: v!("$token"),
            },
            _ => return Err(()),
        })
        // hyper docs: Note: To read the full body, use body::to_bytes or body::aggregate.
    }
}

impl<S: Send + Sync> FromRequest<S, Body> for IntoOp {
    type Rejection = ();
    fn from_request<'a: 'b, 'b>(
        req: Request<Body>,
        _state: &'a S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'b>> {
        Box::pin(std::future::ready(Ok(IntoOp(req))))
    }
}

const SHA256_LEN: usize = 32; // sha256 should be 32 bytes len
const UID_LEN_LIMIT: usize = 32;

fn bytes2hex(bytes: &[u8]) -> Vec<u8> {
    fn to_hex(d: u8) -> u8 {
        match d {
            0..=9 => d + b'0',
            10..=255 => d - 10 + b'a',
        }
    }
    let mut buf = Vec::new();
    for byte in bytes {
        buf.push(to_hex(byte >> 4));
        buf.push(to_hex(byte & 15));
    }
    buf
}

fn hex2bytes<const N: usize>(hex: &[u8]) -> Result<[u8; N], ()> {
    fn from_hex(d: u8) -> Result<u8, ()> {
        Ok(match d {
            b'0'..=b'9' => d - b'0',
            b'a'..=b'f' => d - b'a' + 10,
            _ => return Err(()),
        })
    }
    if hex.len() != N * 2 {
        return Err(());
    }
    let mut ret = [0; N];
    for (i, chunk) in hex.chunks(2).enumerate() {
        ret[i] = (from_hex(chunk[0])? << 4) | from_hex(chunk[1])?;
    }
    Ok(ret)
}

fn json_response<T: IntoResponse>(i: T) -> Response {
    ([(CONTENT_TYPE, "application/json")], i).into_response()
}

async fn post_handler(mut into_op: IntoOp) -> Response {
    const ERR_TOKEN: &str = r#"{"type":"err_token"}"#;
    let op = match into_op.into_op() {
        Ok(v) => v,
        Err(_) => return json_response(r#"{"type":"err_header_invalid"}"#),
    };
    match op {
        Op::Signup { uid, upw, mail } => {
            if uid.len() > UID_LEN_LIMIT {
                return json_response(r#"{"type":"err_uid_too_long"}"#);
            }
            if db_user_r(uid).is_some() {
                return json_response(r#"{"type":"err_uid_exists"}"#);
            }
            let mut upw_buf = [0u8; SHA256_LEN * 2];
            for v in &mut upw_buf[..SHA256_LEN] {
                *v = rand::random();
            }
            let upw = match hex2bytes::<SHA256_LEN>(upw) {
                Ok(v) => v,
                Err(()) => return json_response(r#"{"type":"err_upw_encode"}"#),
            };
            upw_buf[SHA256_LEN..].copy_from_slice(&upw);
            let sha256 = ring::digest::digest(&ring::digest::SHA256, &upw_buf);
            upw_buf[SHA256_LEN..].copy_from_slice(sha256.as_ref());
            db_user_cu(uid, &upw_buf, mail, 64); // TODO: mail vertify
            json_response(r#"{"type":"ok_signup"}"#)
        }
        Op::Login { uid, upw } => {
            // TODO: uid length limit
            const ERR_UID_UPW_WRONG: &str = r#"{"type":"err_uid_upw_wrong"}"#;
            let (upw_correct, _mail, level) = match db_user_r(uid) {
                Some(v) => v,
                None => return json_response(ERR_UID_UPW_WRONG), // TODO: avoid time-side attack
            };
            let mut upw_buf = upw_correct[..SHA256_LEN].to_vec();
            upw_buf.append(&mut ring::test::from_hex(std::str::from_utf8(upw).unwrap()).unwrap());
            assert!(upw_buf.len() == SHA256_LEN * 2);
            let upw_req = ring::digest::digest(&ring::digest::SHA256, &upw_buf);
            if upw_req.as_ref() != &upw_correct[SHA256_LEN..] {
                return json_response(ERR_UID_UPW_WRONG);
            }
            let token = token::current(uid, level);
            json_response(format!(
                r#"{{"type":"ok_login","token":"{token}","level":{level}}}"#
            ))
        }
        Op::Create {
            token,
            desc,
            mime,
            mut body,
        } => {
            let (uid, level) = match token::vertify(token) {
                Ok(v) => v,
                Err(_) => return json_response(ERR_TOKEN),
            };
            // prevent big file in front end, just make a later limit here
            // tokio::fs::write(path, contents)
            while let Some(buf) = body.data().await {
                let buf = match buf {
                    Ok(v) => v,
                    Err(_) => return json_response(r#"{"type":"err_body_read_failed"}"#),
                };

                // if buf.has_remaining() {
                //     bufs.push(buf);
                // }
            }

            // hyper::body::aggregate(body);
            unimplemented!()
        }
        Op::List { token } => {
            let (uid, _level) = match token::vertify(token) {
                Ok(v) => v,
                Err(_) => return Json(ERR_TOKEN).into_response(),
            };
            let mut response = Vec::new();
            response.extend(br#"{"type":"ok_list","files":["#);
            for (fid, size, mut desc, mut mime) in db_data_r_by_user(uid) {
                response.extend(br#"{"fid":"#);
                write!(response, "{fid}").unwrap();
                response.extend(br#","size":"#);
                write!(response, "{size}").unwrap();
                response.extend(br#","desc":""#);
                response.append(&mut desc);
                response.extend(br#"","mime":""#);
                response.append(&mut mime);
                response.extend(br#""},"#);
            }
            if response.last() == Some(&b',') {
                response.pop();
            }
            response.extend(br#"]}"#);
            json_response(response)
        }
    }
}

pub fn dev() {
    let mut buf = ring::test::from_hex(
        std::str::from_utf8(b"2d6fbf923fd5b2ad1bb7d036da1d153137072036d2c48b1c0aea2d234cdd30e3")
            .unwrap(),
    )
    .unwrap();
    dbg!(buf.len());
}

pub fn service() -> Router {
    // TODO: vertify if the trigger fn not register!
    // TODO: user may make request immediately after the server launch, is this sound?
    db_init();
    token::renew_tick();
    Router::new().route(
        "/paste",
        MethodRouter::new()
            .get(|| async { Html((include_page!("page.html") as [_; 1])[0]) })
            .post(post_handler),
    )
}

static TICKER: Lazy<Ticker> = Lazy::new(|| Ticker::new_p8(&[(-1, 0, 0), (-1, 30, 0)]));
pub async fn tick() {
    if !TICKER.tick() {
        return;
    }
    token::renew_tick();
}
