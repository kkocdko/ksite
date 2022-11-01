#![allow(unused)]

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
use crate::{db, include_page, strip_str};
use axum::body::{Body, Bytes, HttpBody};
use axum::extract::rejection::StringRejection;
use axum::extract::{BodyStream, Form, FromRequest, FromRequestParts, Json, Path, RawBody};
use axum::http::header::{self, HeaderMap, HeaderValue};
use axum::http::request::Parts as RequestParts;
use axum::http::{Request, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{MethodRouter, Router};
use ring::digest::Digest;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Display;
use std::future::Future;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
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

static TOKEN_SEED_POOL: [[u8; SHA256_LEN]; 2] = [[0u8; SHA256_LEN]; 2]; // use unsafe to modify
static TOKEN_SEED_IDX: AtomicUsize = AtomicUsize::new(usize::MAX); // use as a mutex lock

// token = sha256(seed + level + uid)

// | token 0 | token 1 |
// -----| token 1 |
//

fn token_gen(uid: &[u8], level: u64) -> Digest {
    let seed = TOKEN_SEED_POOL[TOKEN_SEED_IDX.load(Ordering::Relaxed)];
    let mut buf = Vec::new();
    buf.extend(seed);
    buf.extend(uid);
    buf.extend(level.to_le_bytes());
    ring::digest::digest(&ring::digest::SHA256, &buf)
}

fn token_vertify(uid: &[u8], level: u64, token: &[u8]) -> bool {
    true
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
    Renew {
        uid: &'a [u8],
        level: u64,
        token: &'a [u8],
    },
    // Download {},
    // Upload {},
    // Get {
    //     fid: i64,
    // },
    // List {
    //     uid: &'a [u8],
    //     token: &'a [u8],
    // },
}

struct IntoOp<B: HttpBody + Send + 'static>(Request<B>);

impl<B: HttpBody + Send + 'static> IntoOp<B> {
    fn into_op(&self) -> Result<Op, ()> {
        let headers = self.0.headers();
        macro_rules! value_of {
            ($k:expr) => {{
                match headers.get($k) {
                    Some(v) => v.as_bytes(),
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
            b"renew" => Op::Renew {
                uid: value_of!("$uid"),
                level: std::str::from_utf8(value_of!("$level"))
                    .unwrap()
                    .parse()
                    .unwrap(),
                token: value_of!("$token"),
            },
            _ => return Err(()),
        })
    }
}

impl<S: Send + Sync, B: HttpBody + Send> FromRequest<S, B> for IntoOp<B> {
    type Rejection = ();
    fn from_request<'a: 'b, 'b>(
        req: Request<B>,
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

async fn post_handler<B: HttpBody + Send + 'static>(into_op: IntoOp<B>) -> Response {
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
            let token = token_gen(uid, level);
            let token = BytesToHex(token.as_ref());
            format!(r#"{{"type":"ok_login","token":"{token}","level":{level}}}"#).into_response()
        }
        Op::Renew { uid, level, token } => {
            if !token_vertify(uid, level, token) {
                return r#"{"type":"err_token_invalid"}"#.into_response();
            }
            let token = token_gen(uid, level);
            let token = BytesToHex(token.as_ref());
            format!(r#"{{"type":"ok_renew","token":"{token}"}}"#).into_response()
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
    // TODO： vertify if the trigger fn not register!
    db_init();
    Router::new().route(
        "/paste",
        MethodRouter::new()
            .get(|| async { (include_page!("page.html") as [_; 1])[0] })
            .post(post_handler),
    )
}

pub async fn tick() {
    // update token seed
}
