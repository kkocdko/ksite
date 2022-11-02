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
use axum::body::{Body, Bytes, HttpBody};
use axum::extract::{BodyStream, Form, FromRequest, RawBody};
use axum::http::{Request, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{MethodRouter, Router};
use once_cell::sync::Lazy;
use ring::digest::Digest;
use std::fmt::Display;
use std::future::Future;
use std::mem::{swap, MaybeUninit};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;
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
        (fid INTEGER PRIMARY KEY AUTOINCREMENT, uid BLOB, desc BLOB, mime BLOB)
    "}
    .unwrap();
    // TODO: insert one to avoid 0th file?
}
fn db_user_cu(uid: &[u8], upw: &[u8], mail: &[u8], level: u64) {
    db! {"
        REPLACE INTO paste_user
        VALUES (?1, ?2, ?3, ?4)
    ",[uid, upw, mail, level as i64]}
    .unwrap();
}
fn db_user_r(uid: &[u8]) -> Option<(Vec<u8>, Vec<u8>, u64)> {
    db! {"
        SELECT * FROM paste_user
        WHERE uid = ?
    ", [uid], ^(1, 2, 3)}
    .ok() // TODO: convert i64 to u64?
}
fn db_user_d(uid: &[u8]) {
    db! {"
        DELETE FROM paste_user
        WHERE uid = ?
    ", [uid]}
    .unwrap();
}

// fn db_data_c(uid: &[u8], desc: &[u8], mime: &[u8]) -> i64 {
//     db! {"
//         INSERT INTO paste_data
//         VALUES (NULL, ?1, ?2, ?3)
//     ", [uid, desc, mime], &}
//     .unwrap()
// }
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
// fn db_data_r_by_user(uid: &[u8]) -> Vec<(i64, Vec<u8>, Vec<u8>)> {
//     db!("SELECT * FROM paste_data WHERE uid = ?", [uid], (0, 2, 3)).unwrap()
// }
// fn db_data_d(fid: i64) -> bool {
//     db!("DELETE FROM paste_data WHERE fid = ?", [fid]).is_ok()
// }

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
/// token = `timestamp:second_u64_le + sha256(seed[timestamp] + uid + level:u64_le)`
///
/// The token was regenerated every 30 minutes, and expired after 1 hour. There are 3 seed in the
/// pool, request renew on client should response the next token and the valid timestamp.
///
/// ```norust
/// | 30 min |
/// | --- seed 00 --- |
///          | --- seed 01 --- |
///                   | --- seed 02 --- |
///                            | --- seed 00 --- |
/// ```
mod token {
    use super::SHA256_LEN;
    use ring::digest::Digest;
    use std::fmt::Display;
    use std::future::Future;
    use std::mem::MaybeUninit;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use std::sync::Mutex;
    use std::time::UNIX_EPOCH;

    const TIMESTAMP_LEN: usize = 8; // u64
    const TOKEN_LEN: usize = TIMESTAMP_LEN + SHA256_LEN;
    static mut SEED_POOL: [[u8; SHA256_LEN]; 3] = [[0; SHA256_LEN]; 3];
    static CURRENT_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

    fn gen(seed_idx: usize, uid: &[u8], level: u64) -> Digest {
        let seed = unsafe { SEED_POOL[seed_idx] };
        let mut buf = Vec::new();
        buf.extend(seed);
        buf.extend(uid);
        buf.extend(level.to_le_bytes());
        ring::digest::digest(&ring::digest::SHA256, &buf) // it's hardware accelerated!
    }

    /// Get the seed index (`0`, `1` or `2`) by timestamp, returns `usize::MAX` if the timestamp
    /// was outdated (after 1 hour).
    fn timestamp2idx(timestamp: u64) -> usize {
        let current_timestamp = CURRENT_TIMESTAMP.load(Ordering::Relaxed);
        const LIMIT: u64 = 3600 - 600; // WARNING: avoid edge case?
        if current_timestamp.saturating_sub(timestamp) > LIMIT {
            return usize::MAX; // outdated
        }
        (timestamp % 5400 / 1800) as _
    }

    /// Generate token in current time.
    ///
    /// # Example
    ///
    /// ```
    /// let token: &[u8] = token_current(uid, level).as_ref();
    /// ```
    pub fn current(uid: &[u8], level: u64) -> Digest {
        let seed_idx = timestamp2idx(CURRENT_TIMESTAMP.load(Ordering::Relaxed));
        gen(seed_idx, uid, level)
    }

    pub fn vertify(uid: &[u8], level: u64, token: &[u8]) -> bool {
        let mut timestamp: [u8; 8] = [0; 8];
        timestamp.copy_from_slice(&token[0..TIMESTAMP_LEN]);
        let seed_idx = timestamp2idx(u64::from_le_bytes(timestamp));
        gen(seed_idx, uid, level).as_ref() == token
    }

    pub fn renew_tick() {
        // WARNING: edge case?
        let current_timestamp = UNIX_EPOCH.elapsed().unwrap().as_secs();
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

enum Op<'a> {
    Signup {
        uid: &'a [u8],
        upw: &'a [u8],
        mail: &'a [u8],
    },
    Login {
        uid: &'a [u8],
        upw: &'a [u8],
    },
    // Renew {
    //     uid: &'a [u8],
    //     level: u64,
    //     token: &'a [u8],
    // },
    // Download {
    //     fid: u64,
    //     uid: &'a [u8],
    //     token: &'a [u8],
    // },
    Upload {
        uid: &'a [u8],
        level: u64,
        token: &'a [u8],
        body: Body,
        // stream: BodyStream,
    },
    // Get {
    //     fid: i64,
    // },
    // List {
    //     uid: &'a [u8],
    //     token: &'a [u8],
    // },
}

struct IntoOp(Request<Body>);

impl IntoOp {
    fn into_op(&mut self) -> Result<Op, ()> {
        let mut body = Body::empty();
        swap(self.0.body_mut(), &mut body);
        let headers = self.0.headers();
        macro_rules! value_of {
            ($k:expr) => {{
                match headers.get($k) {
                    Some(v) => v.as_bytes(),
                    None => return Err(()),
                }
            }};
            (:$k:expr) => {{
                match headers.get($k) {
                    Some(v) => v.to_str().or(Err(()))?.parse().or(Err(()))?,
                    None => return Err(()),
                }
            }};
        }
        Ok(match value_of!("$op") {
            b"signup" => Op::Signup {
                uid: value_of!("$uid"),
                upw: value_of!("$upw"),
                mail: value_of!("$mail"),
            },
            b"login" => Op::Login {
                uid: value_of!("$uid"),
                upw: value_of!("$upw"),
            },
            b"upload" => Op::Upload {
                uid: value_of!("$uid"),
                level: value_of!(:"$level"),
                token: value_of!("$token"),
                body,
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
const UID_LEN: usize = 32; // uid should be 32 bytes len

struct BytesToHex<'a>(&'a [u8]);

impl<'a> Display for BytesToHex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn to_hex(d: u8) -> u8 {
            match d {
                0..=9 => d + b'0',
                10..=255 => d - 10 + b'a',
            }
        }
        for byte in self.0 {
            write!(f, "{}", to_hex(byte >> 4) as char)?;
            write!(f, "{}", to_hex(byte & 15) as char)?;
        }
        Ok(())
    }
}

fn hex2bytes(hex: &[u8], bytes: &mut Vec<u8>) {
    unimplemented!()
}

async fn post_handler(mut into_op: IntoOp) -> Response {
    const ERR_TOKEN: &str = r#"{"type":"err_token"}"#;
    let op = match into_op.into_op() {
        Ok(v) => v,
        Err(_) => return r#"{"type":"err_header_invalid"}"#.into_response(),
    };
    match op {
        Op::Signup { uid, upw, mail } => {
            // TODO: uid length limit
            if db_user_r(uid).is_some() {
                return r#"{"type":"err_uid_exists"}"#.into_response();
            }
            dbg!(std::str::from_utf8(upw).unwrap());
            let upw = ring::test::from_hex(std::str::from_utf8(upw).unwrap()).unwrap(); // TODO: change to static convert function
            let mut upw_buf = [0u8; SHA256_LEN * 2];
            for v in &mut upw_buf[..SHA256_LEN] {
                *v = rand::random(); // TODO: use rand::Fill?
            }
            // avoid panic of copy_from_slice
            if upw.len() != SHA256_LEN {
                return r#"{"type":"err_upw_len"}"#.into_response();
            }
            upw_buf[SHA256_LEN..].copy_from_slice(&upw);
            let sha256 = ring::digest::digest(&ring::digest::SHA256, &upw_buf);
            upw_buf[SHA256_LEN..].copy_from_slice(sha256.as_ref());
            db_user_cu(uid, &upw_buf, mail, 64); // TODO: mail vertify
            r#"{"type":"ok_signup"}"#.into_response()
        }
        Op::Login { uid, upw } => {
            // TODO: uid length limit
            const ERR_UID_UPW_WRONG: &str = r#"{"type":"err_uid_upw_wrong"}"#;
            let (upw_correct, _mail, level) = match db_user_r(uid) {
                Some(v) => v,
                None => return ERR_UID_UPW_WRONG.into_response(), // TODO: avoid time-side attack
            };
            let mut upw_buf = upw_correct[..SHA256_LEN].to_vec();
            upw_buf.append(&mut ring::test::from_hex(std::str::from_utf8(upw).unwrap()).unwrap());
            assert!(upw_buf.len() == SHA256_LEN * 2);
            let upw_req = ring::digest::digest(&ring::digest::SHA256, &upw_buf);
            if upw_req.as_ref() != &upw_correct[SHA256_LEN..] {
                return ERR_UID_UPW_WRONG.into_response();
            }
            let token = token::current(uid, level);
            let token = BytesToHex(token.as_ref());
            format!(r#"{{"type":"ok_login","token":"{token}","level":{level}}}"#).into_response()
        }
        Op::Upload {
            uid,
            level,
            token,
            body,
        } => {
            if !token::vertify(uid, level, token) {
                return ERR_TOKEN.into_response();
            }
            unimplemented!()
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
    unimplemented!()
    // unsafe { token_renew_tick() }
}
