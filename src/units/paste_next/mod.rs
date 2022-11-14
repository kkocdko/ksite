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
* 增量更新
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
use crate::{care, db, include_page};
use axum::body::StreamBody;
use axum::body::{Body, Bytes, HttpBody as _};
use axum::extract::{FromRef, FromRequest};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Request};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{MethodRouter, Router};
use futures_core::{Stream};
use hyper::header::{HeaderName, CONTENT_LENGTH};
use once_cell::sync::Lazy;
use std::ffi::OsStr;
use std::future::Future;
use std::io::{Read, Write as _};
use std::mem::swap;
use std::os::unix::prelude::OsStrExt;
use std::path::PathBuf;
use std::pin::Pin;
use std::str::FromStr;
use std::task::Context;
use std::task::Poll;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::io::{self, AsyncRead};
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
    // TODO: insert some entrys to skip id 0~32?
    // TODO: built in guest account?
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
fn db_data_r(fid: u64) -> Option<(u64, Vec<u8>, Vec<u8>, Vec<u8>)> {
    db!("SELECT * FROM paste_data WHERE fid = ?", [fid], ^(1, 2, 3, 4)).ok()
}
fn db_data_r_by_user(uid: &[u8]) -> Vec<(u64, u64, Vec<u8>, Vec<u8>)> {
    db!(
        "SELECT * FROM paste_data WHERE uid = ?",
        [uid],
        (0, 1, 3, 4)
    )
    .unwrap()
}
fn db_data_d(fid: u64) -> bool {
    db!("DELETE FROM paste_data WHERE fid = ?", [fid]).is_ok()
}
const DEFAULT_USER_LEVEL: u8 = 64;
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
    Login {
        uid: &'a [u8],
        upw: &'a [u8],
    },
    /// Change user profile.
    // Profile {
    //     upw: &'a [u8],
    //     mail: &'a [u8],
    //     level: &'a [u8],
    //     token: &'a [u8],
    // },
    /// Get the files list.
    List {
        token: &'a [u8],
    },
    /// Create a file.
    Create {
        token: &'a [u8],
        size: &'a [u8],
        desc: &'a [u8],
        mime: &'a [u8],
        body: Body,
    },
    // Download a file.
    Download {
        token: &'a [u8],
        fid: &'a [u8],
    },
    // Delete a file.
    Delete {
        token: &'a [u8],
        fid: &'a [u8],
    },
    // Update {},
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
        Ok(match v!(OP_) {
            b"signup" => Op::Signup {
                uid: v!(UID_),
                upw: v!(UPW_),
                mail: v!(MAIL_),
            },
            b"login" => Op::Login {
                uid: v!(UID_),
                upw: v!(UPW_),
            },
            b"create" => Op::Create {
                token: v!(TOKEN_),
                size: v!(SIZE_),
                desc: v!(DESC_),
                mime: v!(MIME_),
                body,
            },
            b"download" => Op::Download {
                token: v!(TOKEN_),
                fid: v!(FID_),
            },
            b"delete" => Op::Delete {
                token: v!(TOKEN_),
                fid: v!(FID_),
            },
            b"list" => Op::List { token: v!(TOKEN_) },
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

fn get_size_limit(level: u8) -> usize {
    const MIB: usize = 1024 * 1024;
    match level {
        128 => 16 * MIB,
        64 => 2 * MIB,
        _ => 0,
    }
}

fn json_response<T: IntoResponse>(i: T) -> Response {
    ([(CONTENT_TYPE, "application/json")], i).into_response()
}

fn fid_to_path(fid: &[u8]) -> PathBuf {
    static STORAGE_ROOT: Lazy<PathBuf> = Lazy::new(|| {
        let mut p = std::env::current_exe().unwrap();
        p.set_file_name("data");
        p.push("paste");
        p.push("storage");
        if !p.exists() {
            std::fs::create_dir_all(&p).unwrap();
        }
        p
    });
    STORAGE_ROOT.join(OsStr::from_bytes(fid))
    // TODO: https://docs.ceph.com/en/quincy/cephfs/index.html
}

fn parse_slice<T: FromStr>(i: &[u8]) -> Result<T, ()> {
    std::str::from_utf8(i).or(Err(()))?.parse().or(Err(()))
}

trait FromBuf {
    fn from_buf(buf: impl Into<Vec<u8>>) -> Self;
}
impl FromBuf for HeaderValue {
    fn from_buf(buf: impl Into<Vec<u8>>) -> Self {
        let buf = buf.into();
        // TODO: use unsafe here?
        Self::from_maybe_shared(Bytes::from(buf)).unwrap()
    }
}
// impl FromString for HeaderName {
//     fn from_string(s: String) -> Self {
//         // TODO: use unsafe here?
//         // HeaderValue::from_string(s)
//         // Self::fr(Bytes::from(s)).unwrap()
//     }
// }

macro_rules! def_const {
    ($k:ident) => {
        const $k: &'static str = {
            const fn lower_case_const<const N: usize>(v: &[u8]) -> [u8; N] {
                let mut ret = [0; N];
                let mut i = 0;
                while i < N {
                    ret[i] = v[i].to_ascii_lowercase();
                    i += 1;
                }
                ret
            }
            const UPPER: &'static str = stringify!($k);
            const LOWER: [u8; UPPER.len()] = lower_case_const(UPPER.as_bytes());
            unsafe { std::str::from_utf8_unchecked(&LOWER) }
        };
    };
}
// header names
def_const!(OP_);
def_const!(UID_);
def_const!(UPW_);
def_const!(TYPE_);
def_const!(MAIL_);
def_const!(TOKEN_);
def_const!(SIZE_);
def_const!(LEVEL_);
def_const!(DESC_);
def_const!(MIME_);
def_const!(FID_);
// ok types
def_const!(OK_DEFAULT);
// err types
def_const!(ERR_TOKEN);
def_const!(ERR_UID_UPW);
def_const!(ERR_UPW_DECODE);
def_const!(ERR_HEADER_INVALID);
def_const!(ERR_BODY_READ);
def_const!(ERR_BODY_SIZE);
def_const!(ERR_UID_EXISTS);
def_const!(ERR_UID_TOO_LONG);
def_const!(ERR_FILE_NOT_FOUND_OR_DENY);
def_const!(ERR_SERVER_INNER);

trait CastErr<T> {
    fn cast_err(self, e: &'static str) -> Result<T, Response>;
}
impl<T, E> CastErr<T> for Result<T, E> {
    fn cast_err(self, e: &'static str) -> Result<T, Response> {
        self.map_err(|_| [(TYPE_, e)].into_response())
    }
}
impl<T> CastErr<T> for Option<T> {
    fn cast_err(self, e: &'static str) -> Result<T, Response> {
        self.ok_or_else(|| [(TYPE_, e)].into_response())
    }
}
impl CastErr<()> for bool {
    fn cast_err(self, e: &'static str) -> Result<(), Response> {
        match self {
            true => Ok(()),
            false => Err([(TYPE_, e)].into_response()),
        }
    }
}

async fn post_handler(mut into_op: IntoOp) -> Result<Response, Response> {
    let op = into_op.into_op().cast_err(ERR_HEADER_INVALID)?;
    match op {
        Op::Signup { uid, upw, mail } => {
            (uid.len() <= UID_LEN_LIMIT).cast_err(ERR_UID_TOO_LONG)?;
            (db_user_r(uid).is_none()).cast_err(ERR_UID_EXISTS)?;
            // salt: [0, SHA256_LEN), content: [SHA256_LEN, 2 * SHA256_LEN)
            let mut upw_buf = [0u8; SHA256_LEN * 2];
            for v in &mut upw_buf[..SHA256_LEN] {
                *v = rand::random(); // salt
            }
            let upw = hex2bytes::<SHA256_LEN>(upw).cast_err(ERR_UPW_DECODE)?;
            upw_buf[SHA256_LEN..].copy_from_slice(&upw);
            let sha256 = ring::digest::digest(&ring::digest::SHA256, &upw_buf);
            upw_buf[SHA256_LEN..].copy_from_slice(sha256.as_ref());
            db_user_cu(uid, &upw_buf, mail, 64); // TODO: mail vertify
            Ok([
                (TYPE_, HeaderValue::from_static(OK_DEFAULT)),
                (LEVEL_, HeaderValue::from(DEFAULT_USER_LEVEL as u16)), // u8 is ambiguity
            ]
            .into_response())
        }
        Op::Login { uid, upw } => {
            // TODO: uid length limit
            let (upw_correct, _mail, level) = db_user_r(uid).cast_err(ERR_UID_UPW)?; // TODO: avoid time-side attack
            let mut upw_buf = [0u8; SHA256_LEN * 2];
            upw_buf[..SHA256_LEN].copy_from_slice(&upw_correct[..SHA256_LEN]); // salt
            let upw_decoded = hex2bytes::<SHA256_LEN>(upw).cast_err(ERR_UPW_DECODE)?;
            upw_buf[SHA256_LEN..].copy_from_slice(&upw_decoded);
            let upw_req = ring::digest::digest(&ring::digest::SHA256, &upw_buf);
            (upw_req.as_ref() == &upw_correct[SHA256_LEN..]).cast_err(ERR_UID_UPW)?;
            let token = token::current(uid, level);
            Ok([
                (TYPE_, HeaderValue::from_static(OK_DEFAULT)),
                (TOKEN_, HeaderValue::from_buf(token)),
                (LEVEL_, HeaderValue::from(level as u16)),
            ]
            .into_response())
        }
        Op::Create {
            token,
            size,
            desc,
            mime,
            mut body,
        } => {
            // TODO: https://github.com/tokio-rs/axum/blob/main/examples/stream-to-file/src/main.rs
            let (uid, level) = token::vertify(token).cast_err(ERR_TOKEN)?;
            // prevent big file in front end, just make a later limit here
            let size_limit = get_size_limit(level);
            let expect_size = parse_slice::<u64>(size).cast_err(ERR_HEADER_INVALID)?;
            let fid = db_data_c(expect_size, uid, desc, mime);
            let p = fid_to_path(fid.to_string().as_bytes());
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&p)
                .await
                .cast_err(ERR_SERVER_INNER)?;
            let mut size = 0;
            while let Some(buf) = body.data().await {
                let buf = buf.cast_err(ERR_BODY_READ)?;
                size += buf.len();
                (size <= size_limit).cast_err(ERR_BODY_SIZE)?;
                care!(file.write(&buf).await).cast_err(ERR_SERVER_INNER)?;
            }
            if size as u64 != expect_size {
                db_data_d(fid);
                file.set_len(0).await.ok();
                file.shutdown().await.ok(); // or flush?
                drop(file);
                care!(tokio::fs::remove_file(&p).await).ok(); // ignore inner error?
                false.cast_err(ERR_BODY_SIZE)?;
            }
            Ok([
                (TYPE_, HeaderValue::from_static(OK_DEFAULT)),
                (FID_, HeaderValue::from(fid)),
            ]
            .into_response())
        }
        Op::Download { token, fid } => {
            let (uid, _level) = token::vertify(token).cast_err(ERR_TOKEN)?;
            let fid_u64 = parse_slice::<u64>(fid).cast_err(ERR_HEADER_INVALID)?;
            let (size, owner_uid, desc, mime) =
                db_data_r(fid_u64).cast_err(ERR_FILE_NOT_FOUND_OR_DENY)?;
            (uid == owner_uid).cast_err(ERR_FILE_NOT_FOUND_OR_DENY)?;
            let p = fid_to_path(fid);
            let file = File::open(p).await.unwrap();
            let body = StreamBody::new(FileStream(file));
            let mut response = body.into_response();
            let headers = response.headers_mut();
            headers.insert(CONTENT_LENGTH, HeaderValue::from(size));
            headers.insert(TYPE_, HeaderValue::from_static(OK_DEFAULT));
            headers.insert(DESC_, HeaderValue::from_buf(desc));
            headers.insert(MIME_, HeaderValue::from_buf(mime));
            Ok(response)
            // https://docs.rs/axum-extra/latest/src/axum_extra/body/async_read_body.rs.html#15-53
        }
        Op::Delete { token, fid } => {
            let (uid, _level) = token::vertify(token).cast_err(ERR_TOKEN)?;
            let fid_u64 = parse_slice::<u64>(fid).cast_err(ERR_HEADER_INVALID)?;
            let (_size, owner_uid, _desc, _mime) =
                db_data_r(fid_u64).cast_err(ERR_FILE_NOT_FOUND_OR_DENY)?;
            (uid == owner_uid).cast_err(ERR_FILE_NOT_FOUND_OR_DENY)?;
            let p = fid_to_path(fid);
            db_data_d(fid_u64)
                .cast_err(ERR_FILE_NOT_FOUND_OR_DENY)
                .unwrap(); // illegal state, unwrap here
            tokio::fs::remove_file(p).await.unwrap();
            Ok([(TYPE_, HeaderValue::from_static(OK_DEFAULT))].into_response())
        }
        Op::List { token } => {
            let (uid, _level) = token::vertify(token).cast_err(ERR_TOKEN)?;
            let list = db_data_r_by_user(uid);
            let list_is_empty = list.is_empty();
            let mut body = Vec::<u8>::new(); // TODO: set capacity for performance
            for (fid, size, mut desc, mut mime) in list {
                write!(body, "fid:{fid}\nsize:{size}\ndesc:").unwrap();
                body.append(&mut desc);
                body.extend(b"\nmime:");
                body.append(&mut mime);
                body.push(b'\n');
                body.push(b'\n');
            }
            if !list_is_empty {
                body.pop(); // depends on a newline at the end of file!
            }
            let mut response = body.into_response();
            let headers = response.headers_mut();
            headers.insert(TYPE_, HeaderValue::from_static(OK_DEFAULT));
            Ok(response)
        }
    }
}

// https://docs.rs/axum-extra/latest/axum_extra/

struct FileStream(File);
impl Stream for FileStream {
    type Item = Result<Vec<u8>, io::Error>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut buf = Vec::with_capacity(4 * 1024);
        let mut fut = self.0.read_buf(&mut buf);
        let fut = unsafe {
            // safety: todo
            Pin::new_unchecked(&mut fut)
        };
        if 0 == futures_core::ready!(fut.poll(cx))? {
            return Poll::Ready(None);
        }
        Poll::Ready(Some(Ok(buf)))
    }
}

pub async fn dev() {
    dbg!(ERR_TOKEN);
    // let mut file = tokio::fs::OpenOptions::new()
    //     // .create(true)
    //     .create(true)
    //     .write(true)
    //     .open("/home/kkocdko/misc/code/ksite/target/debug/data/paste/storage/2")
    //     .await
    //     .unwrap();
    // dbg!(std::str::from_utf8(
    //     &hyper::body::to_bytes(
    //         Result::<Response, Response>::Err("err".into_response())
    //             .into_response()
    //             .into_body()
    //     )
    //     .await
    //     .unwrap()
    // )
    // .unwrap());
}

pub fn service() -> Router {
    // TODO: vertify if the trigger fn not register!
    // TODO: user may make request immediately after the server launch, is this sound?
    // dbg!(STORAGE_ROOT.to_str());
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
