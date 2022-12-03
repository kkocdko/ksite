/*
剪贴板 & 轻量文件存储

features:
* 账户登录
* 强加密
* 文件分享
* 轻量，快速，不使用框架的界面

details:
* fid 不要超过 i64（测试 u64 的可能性？）
* fid 在前端从 str 转成 int
* fid 生成完美哈希，防止用户争抢短 id
* 写入原始内容，前端预览时自行处理转义
* 不作服务端解密，前端边解密边下载

evolution:
* 防止基于时间的侧信道攻击
* 防止重放攻击
* 修改用户名，不对称算力验证防攻击
* 邮箱关联
* 断点续传
* 页面缓存
* 会员制
* 增量更新（虽然难度很大）

todo:
* [x] 数据库结构
* [x] 账户登录
* [x] 高性能 token 模块
* [x] 基础界面
* [x] 流式传输（后端）
* [x] 全局加密化（后端）
* [ ] 全局加密化（前端）
* [ ] 限制每个用户创建的文件数量
* [ ] 完成零散的 todo 项目

sessions?
全部 public，未分享的用用户密码加密?
读写本地文件账户登录，加密
原生js糊的轻量的界面，借鉴一点点 react 之类的东西
在用户之间分享，转移所有权
尽量优化性能
内部用数字存储 fid，文件名和路径也是数字？
账户创建 file 的速度限制。
区分创建与插入？评估性能影响

https://github.com/lettre/lettre
https://www.runoob.com/sqlite/sqlite-intro.html
https://github.com/su18/wooyun-drops/blob/b2a5416/papers/%E5%8A%A0%E7%9B%90hash%E4%BF%9D%E5%AD%98%E5%AF%86%E7%A0%81%E7%9A%84%E6%AD%A3%E7%A1%AE%E6%96%B9%E5%BC%8F.md
https://docs.rs/rustls/latest/rustls/internal/msgs/enums/enum.HashAlgorithm.html#variant.SHA256
https://docs.rs/ring/latest/ring/digest/fn.digest.html
file:///home/kkocdko/misc/Markdown_1666085837234.html

-----

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
use self::consts::*;
use self::misc::*;
use crate::include_page;
use crate::ticker::Ticker;
use axum::body::{Body, Bytes, HttpBody};
use axum::extract::FromRequest;
use axum::http::header::{HeaderMap, HeaderValue, CONTENT_LENGTH};
use axum::http::Request;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{MethodRouter, Router};
use hyper::header::CACHE_CONTROL;
use once_cell::sync::Lazy;
use rand::{thread_rng, Fill};
use ring::digest::Digest;
use std::future::Future;
use std::io::Write;
use std::mem::swap;
use std::path::PathBuf;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::task::{Context, Poll};
use std::time::UNIX_EPOCH;
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};

/// Token Module
///
/// The token was regenerated every 30 minutes, and expired after 1 hour. There are 3 seed in the
/// pool, request renew on client should response the next token and the valid timestamp.
///
/// ```norust
/// timestamp: u32 = minutes, ulv: u8, uid: &[u8]
/// token: &[u8] =
///     hash(seed[timestamp] + ulv:u8 + uid)
///     + timestamp:u32_le_bytes
///     + ulv:u8
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
    use super::*;

    static mut SEED_POOL: [[u8; 32]; 3] = [[0; 32]; 3];
    static CURRENT_TIMESTAMP: AtomicU32 = AtomicU32::new(0);

    fn hash(seed_idx: usize, ulv: u8, uid: &[u8]) -> Digest {
        let seed = unsafe { SEED_POOL[seed_idx] };
        let mut buf = Vec::new(); // TODO: avoid dynamic memory
        buf.extend(seed);
        buf.push(ulv);
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
    pub fn current(uid: &[u8], ulv: u8) -> String {
        let timestamp = CURRENT_TIMESTAMP.load(Ordering::Relaxed);
        let seed_idx = timestamp2idx(timestamp);
        let mut buf = Vec::new();
        buf.extend(hash(seed_idx, ulv, uid).as_ref());
        buf.extend(timestamp.to_le_bytes());
        buf.extend(ulv.to_le_bytes());
        buf = bytes2hex(&buf);
        buf.push(b'.');
        buf.extend(uid);
        String::from_utf8(buf).unwrap()
    }

    /// Extract (uid, ulv).
    pub fn vertify(token: &[u8]) -> Result<(&[u8], u8), ()> {
        if token.len() < 76 {
            return Err(()); // too short
        }
        let sha256: [u8; 32] = hex2bytes(&token[..64])?;
        let timestamp = u32::from_le_bytes(hex2bytes(&token[64..72])?);
        let ulv = hex2bytes::<1>(&token[72..74])?[0];
        let uid = &token[75..];
        let seed_idx = timestamp2idx(timestamp);
        match hash(seed_idx, ulv, uid).as_ref() == sha256 {
            true => Ok((uid, ulv)),
            false => Err(()),
        }
        // In export interface, we use (uid, ulv); in this module, due to the flexibility, we use
        // the token format that uid is behind the ulv.
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

mod db {
    #![allow(clippy::type_complexity)]

    use crate::db;

    /*
    d -> database, c -> client
    d:uid = c:uid = user id text
    d:mail = user email
    d:ulv = user level (admin / vip / banned / normal)
    upw_raw = raw user password text
    c:upw = sha256(`paste` + c:uid + upw_raw)
    salt = rand([u8; 32])
    d:upw = sha256(salt + c:upw)
    d:fid = u64 file id
    c:fpw = rand([u8; 32])
    file_raw = raw file data
    file = file data storaged at server
    file = file_raw.aes256(c:fpw)
    d:cap = file.length
    meta_inner = json { size = file_raw.length, desc, mime }
    d:meta = json { fpw = c:fpw.aes256(c:upw), inner = meta_inner.aes256(c:fpw) }
    */
    pub fn init() {
        db! {"
            CREATE TABLE IF NOT EXISTS paste_user
            (uid BLOB PRIMARY KEY, upw BLOB, mail BLOB, ulv INTEGER)
        "}
        .unwrap();
        db! {"
            CREATE TABLE IF NOT EXISTS paste_data
            (fid INTEGER PRIMARY KEY AUTOINCREMENT, uid BLOB, cap INTEGER, meta BLOB)
        "}
        .unwrap();
        // TODO: built in guest account?
    }

    pub fn user_cu(uid: &[u8], upw: &[u8], mail: &[u8], ulv: u8) {
        db! {"
            REPLACE INTO paste_user
            VALUES (?1, ?2, ?3, ?4)
        ",[uid, upw, mail, ulv as i64]}
        .unwrap();
    }

    pub fn user_r(uid: &[u8]) -> Option<(Vec<u8>, Vec<u8>, u8)> {
        db! {"
            SELECT * FROM paste_user
            WHERE uid = ?
        ", [uid], ^(1, 2, 3)}
        .ok()
    }

    pub fn user_r_ulv(uid: &[u8]) -> Option<(u8,)> {
        db! {"
            SELECT ulv FROM paste_user
            WHERE uid = ?
        ", [uid], ^(0)}
        .ok()
    }

    pub fn user_d(uid: &[u8]) {
        db! {"
            DELETE FROM paste_user
            WHERE uid = ?
        ", [uid]}
        .unwrap();
    }

    pub fn data_c(uid: &[u8], cap: u64, meta: &[u8]) -> u64 {
        db! {"
            INSERT INTO paste_data
            VALUES (NULL, ?1, ?2, ?3)
        ", [uid, cap, meta], &}
        .unwrap() as _
    }

    pub fn data_r(fid: u64) -> Option<(Vec<u8>, u64, Vec<u8>)> {
        db! {"
            SELECT * FROM paste_data
            WHERE fid = ?
        ", [fid], ^(1, 2, 3)}
        .ok()
    }

    pub fn data_r_by_user(uid: &[u8]) -> Vec<(u64, u64, Vec<u8>)> {
        db! {"
            SELECT * FROM paste_data
            WHERE uid = ?
        ", [uid], (0, 2, 3)}
        .unwrap()
    }

    pub fn data_u(fid: u64, cap: u64, meta: &[u8]) {
        db! {"
            UPDATE paste_data
            SET cap = ?2, meta = ?3
            WHERE fid = ?1
        ", [fid, cap, meta]}
        .unwrap();
    }

    pub fn data_d(fid: u64) {
        db! {"
            DELETE FROM paste_data
            WHERE fid = ?
        ", [fid]}
        .unwrap();
    }
}

mod consts {
    /// Define a const str mark.
    ///
    /// ```
    /// // write this
    /// def_str!(FOO_BAR);
    /// // expand to
    /// const FOO_BAR: &'static str = "foo_bar";
    /// ```
    macro_rules! def_str {
        ($k:ident) => {
            pub const $k: &'static str = {
                const fn lower_case_const<const N: usize>(v: &[u8]) -> [u8; N] {
                    let mut ret = [0; N];
                    let mut i = 0;
                    while i < N {
                        ret[i] = v[i].to_ascii_lowercase();
                        i += 1;
                    }
                    ret
                }
                const UPPER: &[u8] = stringify!($k).as_bytes();
                const LOWER: [u8; UPPER.len()] = lower_case_const(UPPER);
                unsafe { std::str::from_utf8_unchecked(&LOWER) }
            };
        };
    }

    // header names
    def_str!(OP_);
    def_str!(TYPE_);
    def_str!(TOKEN_);
    def_str!(UID_);
    def_str!(UPW_);
    def_str!(ULV_);
    def_str!(MAIL_);
    def_str!(FID_);
    def_str!(FPW_);
    def_str!(META_);
    def_str!(LIMIT_);

    // ok types
    def_str!(OK_DEFAULT);

    // err types
    def_str!(ERR_TOKEN);
    def_str!(ERR_UID_UPW);
    def_str!(ERR_UID_EXISTS);
    def_str!(ERR_UID_TOO_LONG);
    def_str!(ERR_UPW_DECODE);
    def_str!(ERR_HEADER_INVALID);
    def_str!(ERR_BODY_READ);
    def_str!(ERR_SIZE_LIMIT);
    def_str!(ERR_FILE_NOT_FOUND_OR_DENY);
    def_str!(ERR_SERVER_INNER);

    // others
    pub const ULV_DEACTIVED: u8 = 7;
    pub const ULV_GUEST: u8 = 15;
    pub const ULV_NORMAL: u8 = 31;
    pub const ULV_VIP: u8 = 63;
    pub const ULV_ADMIN: u8 = 255;
    pub const SHA256_LEN: usize = 32; // sha256 should be 32 bytes len
    pub const UID_LEN_LIMIT: usize = 32;
}

mod misc {
    use tokio::io::AsyncReadExt;

    use super::*;

    pub fn bytes2hex(bytes: &[u8]) -> Vec<u8> {
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

    pub fn hex2bytes<const N: usize>(hex: &[u8]) -> Result<[u8; N], ()> {
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

    // increase a little, because AES will cause size inflate
    // TODO: what is the best value of increasing?
    // TODO: tweak limit bound.
    const MIB_INFLATE: usize = 1024 * (1024 + 128);

    pub const fn ulv_trans_limit(ulv: u8) -> usize {
        match ulv {
            ULV_GUEST => 16 * MIB_INFLATE,
            ULV_NORMAL => 32 * MIB_INFLATE,
            ULV_VIP => 512 * MIB_INFLATE,
            ULV_ADMIN => usize::MAX,
            _ => 0,
        }
    }

    pub const fn ulv_share_limit(ulv: u8) -> usize {
        match ulv {
            ULV_VIP => 384 * MIB_INFLATE,
            ULV_ADMIN => usize::MAX,
            _ => 0,
        }
    }

    pub fn fid_to_path(fid: &[u8]) -> PathBuf {
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
        STORAGE_ROOT.join(std::str::from_utf8(fid).unwrap())
        // TODO: https://docs.ceph.com/en/quincy/cephfs/index.html
    }

    /// Convert `&[u8]` -> `&str` -> `T`.
    pub fn parse_slice<T: FromStr>(i: &[u8]) -> Result<T, ()> {
        std::str::from_utf8(i).or(Err(()))?.parse().or(Err(()))
    }

    /// Because `HeaderValue` does not provide a constructor from `Vec<u8>`.
    pub trait FromBuf {
        fn from_buf(buf: impl Into<Vec<u8>>) -> Self;
    }

    impl FromBuf for HeaderValue {
        fn from_buf(buf: impl Into<Vec<u8>>) -> Self {
            let buf: Vec<u8> = buf.into();
            Self::from_maybe_shared(Bytes::from(buf)).unwrap()
        }
    }

    /// Convert `Result`, `Option` and `bool` into `Result<T, Response>`.
    pub trait CastErr<T> {
        /// Produce `Result<T, Response>` for handlers.
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
        /// Like a `assert!()` macro but returns an `Response` with error message.
        fn cast_err(self, e: &'static str) -> Result<(), Response> {
            match self {
                true => Ok(()),
                false => Err([(TYPE_, e)].into_response()),
            }
        }
    }

    /// A warpper for tokio's `File` which implemented the stream read.
    type FileStreamFut = Pin<Box<dyn Future<Output = (Vec<u8>, io::Result<usize>, File)> + Send>>;

    pub struct FileStream {
        fut: FileStreamFut,
    }

    impl FileStream {
        const BUF_CAPACITY: usize = 64 * 1024;

        pub fn new(file: File) -> Self {
            Self {
                fut: Self::make_fut(file),
            }
        }

        fn make_fut(mut file: File) -> FileStreamFut {
            Box::pin(async {
                let mut buf = Vec::with_capacity(FileStream::BUF_CAPACITY);
                let result = file.read_buf(&mut buf).await;
                (buf, result, file)
            })
        }
    }

    impl HttpBody for FileStream {
        type Data = Bytes;
        type Error = io::Error;

        fn poll_data(
            self: Pin<&mut Self>,
            cx: &mut Context,
        ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
            let mut fut = unsafe { Pin::new_unchecked(&mut self.get_mut().fut) };
            let (buf, result, file) = futures_core::ready!(Pin::new(&mut fut).poll(cx));
            match result {
                Err(e) => Poll::Ready(Some(Err(e))),
                Ok(0) => Poll::Ready(None),
                Ok(_) => {
                    *fut = Self::make_fut(file);
                    Poll::Ready(Some(Ok(buf.into())))
                }
            }
        }

        fn poll_trailers(
            self: Pin<&mut Self>,
            _: &mut Context,
        ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
            Poll::Ready(Ok(None))
        }
    }

    impl IntoResponse for FileStream {
        fn into_response(self) -> Response {
            Response::new(axum::body::boxed(self))
        }
    }

    /// Extract to an uninited `Op<'static>`.
    impl<S> FromRequest<S, Body> for Op<'static> {
        type Rejection = ();
        fn from_request<'a: 'b, 'b>(
            req: Request<Body>,
            _: &'a S,
        ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'b>> {
            Box::pin(std::future::ready(Ok(Op::Uninit { req })))
        }
    }

    pub async fn write_body_to_file(
        mut body: Body,
        file: &mut File,
        limit: usize,
    ) -> Result<usize, Response> {
        let mut remain = limit;
        while let Some(buf) = body.data().await {
            let buf = buf.cast_err(ERR_BODY_READ)?;
            let len = buf.len();
            (remain >= len).cast_err(ERR_SIZE_LIMIT)?;
            remain -= len;
            file.write_all(&buf).await.unwrap();
        }
        Ok(limit - remain)
    }

    impl Op<'static> {
        pub fn init(&mut self) -> Result<Op, ()> {
            let Op::Uninit { req } = self else { unreachable!() };
            let mut body = Body::empty(); // TODO: optimize unnecessary body extact
            swap(req.body_mut(), &mut body);
            let headers = req.headers();
            // TODO: limit the length of META?
            let v = |k: &str| headers.get(k).map_or(Err(()), |o| Ok(o.as_bytes()));
            Ok(match v(OP_)? {
                b"signup" => Op::Signup {
                    uid: v(UID_)?,
                    upw: v(UPW_)?,
                    mail: v(MAIL_)?,
                },
                b"login" => Op::Login {
                    uid: v(UID_)?,
                    upw: v(UPW_)?,
                },
                b"create" => Op::Create {
                    token: v(TOKEN_)?,
                    meta: v(META_)?,
                    body,
                },
                b"replace" => Op::Replace {
                    token: v(TOKEN_)?,
                    fid: v(FID_)?,
                    meta: v(META_)?,
                    body,
                },
                b"delete" => Op::Delete {
                    token: v(TOKEN_)?,
                    fid: v(FID_)?,
                },
                b"list" => Op::List { token: v(TOKEN_)? },
                b"download" => Op::Download {
                    token: v(TOKEN_)?,
                    fid: v(FID_)?,
                    meta: v(META_)?,
                    limit: v(LIMIT_)?,
                },
                _ => return Err(()),
            })
            // hyper docs: Note: To read the full body, use body::to_bytes or body::aggregate.
        }
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

    // /// Convert fid, hashed -> raw
    // fn fid_h2r(i: u64) -> u64 {
    //     i
    // }

    // /// Convert fid, raw -> hashed
    // fn fid_r2h(i: u64) -> u64 {
    //     i
    // }

    // /// Promised that every comparison cost the same time.
    // fn slow_eq(a: &[u8], b: &[u8]) -> bool {
    //     if a.len() != b.len() {
    //         return false; // this is safe because everyone knows the hash len
    //     }
    //     let mut diff = 0;
    //     for i in 0..b.len() {
    //         diff |= a[i] ^ b[i];
    //     }
    //     diff == 0
    // }
}

/// Operation from client requests.
enum Op<'a> {
    /// Once the `Op` created, it's in `Uninit` state and needs a `op.init()`
    Uninit { req: Request<Body> },
    /// Create a new user.
    Signup {
        uid: &'a [u8],
        upw: &'a [u8],
        mail: &'a [u8],
    },
    /// User login or token renew.
    Login { uid: &'a [u8], upw: &'a [u8] },
    // TODO: Change user profile.
    // TODO: User volumn limit.
    /// Create a file.
    Create {
        token: &'a [u8],
        meta: &'a [u8],
        body: Body,
    },
    /// Update a file by replace.
    Replace {
        token: &'a [u8],
        fid: &'a [u8],
        meta: &'a [u8],
        body: Body,
    },
    // Update a file by modify changes.
    // Modify {},
    /// Delete a file.
    Delete { token: &'a [u8], fid: &'a [u8] },
    /// Get the files list.
    List { token: &'a [u8] },
    /// Download a file.
    Download {
        token: &'a [u8],
        fid: &'a [u8],
        meta: &'a [u8],
        /// Size limit in bytes, desktop and mobile' s limit may be different.
        limit: &'a [u8],
    },
}

async fn post_handler(mut op: Op<'static>) -> Result<Response, Response> {
    match op.init().cast_err(ERR_HEADER_INVALID)? {
        Op::Uninit { .. } => panic!("op is uninit"),

        Op::Signup { uid, upw, mail } => {
            (uid.len() <= UID_LEN_LIMIT).cast_err(ERR_UID_TOO_LONG)?;
            (db::user_r(uid).is_none()).cast_err(ERR_UID_EXISTS)?;
            // layout = salt: [0, SHA256_LEN), content: [SHA256_LEN, 2 * SHA256_LEN)
            let mut upw_buf = [0u8; SHA256_LEN * 2];
            upw_buf[..SHA256_LEN].try_fill(&mut thread_rng()).unwrap(); // salt
            let upw = hex2bytes::<SHA256_LEN>(upw).cast_err(ERR_UPW_DECODE)?;
            upw_buf[SHA256_LEN..].copy_from_slice(&upw);
            let sha256 = ring::digest::digest(&ring::digest::SHA256, &upw_buf);
            upw_buf[SHA256_LEN..].copy_from_slice(sha256.as_ref());
            db::user_cu(uid, &upw_buf, mail, 64); // TODO: mail vertify
            Ok([
                (TYPE_, HeaderValue::from_static(OK_DEFAULT)),
                (ULV_, HeaderValue::from(ULV_NORMAL as u16)), // u8 is ambiguity
            ]
            .into_response())
        }

        Op::Login { uid, upw } => {
            // TODO: uid length limit
            // TODO: avoid time-side attack?
            let (upw_correct, _mail, ulv) = db::user_r(uid).cast_err(ERR_UID_UPW)?;
            let mut upw_buf = [0u8; SHA256_LEN * 2];
            upw_buf[..SHA256_LEN].copy_from_slice(&upw_correct[..SHA256_LEN]); // salt
            let upw_decoded = hex2bytes::<SHA256_LEN>(upw).cast_err(ERR_UPW_DECODE)?;
            upw_buf[SHA256_LEN..].copy_from_slice(&upw_decoded);
            let upw_req = ring::digest::digest(&ring::digest::SHA256, &upw_buf);
            (upw_req.as_ref() == &upw_correct[SHA256_LEN..]).cast_err(ERR_UID_UPW)?;
            let token = token::current(uid, ulv);
            Ok([
                (TYPE_, HeaderValue::from_static(OK_DEFAULT)),
                (TOKEN_, HeaderValue::from_buf(token)),
                (ULV_, HeaderValue::from(ulv as u16)),
            ]
            .into_response())
        }

        Op::Create { token, meta, body } => {
            let (uid, ulv) = token::vertify(token).cast_err(ERR_TOKEN)?;
            let fid_u64 = db::data_c(uid, 0, b"");
            let p = fid_to_path(fid_u64.to_string().as_bytes());
            let mut file = File::create(&p).await.unwrap();
            let limit_by_ulv = ulv_trans_limit(ulv); // prevent big file in front end, just a later limit here
            if let Err(err) = write_body_to_file(body, &mut file, limit_by_ulv).await {
                db::data_d(fid_u64);
                file.set_len(0).await.ok();
                file.shutdown().await.ok(); // or flush?
                drop(file);
                tokio::fs::remove_file(&p).await.unwrap(); // ignore inner error?
                return Err(err);
            }
            // TODO: use buffer len?
            db::data_u(fid_u64, file.metadata().await.unwrap().len(), meta);
            Ok([
                (TYPE_, HeaderValue::from_static(OK_DEFAULT)),
                (FID_, HeaderValue::from(fid_u64)),
            ]
            .into_response())
        }

        Op::Replace {
            token,
            fid,
            meta,
            body,
        } => {
            let (uid, ulv) = token::vertify(token).cast_err(ERR_TOKEN)?;
            let fid_u64 = parse_slice::<u64>(fid).cast_err(ERR_HEADER_INVALID)?;
            let (owner_uid, cap, _meta) =
                db::data_r(fid_u64).cast_err(ERR_FILE_NOT_FOUND_OR_DENY)?;
            (uid == owner_uid).cast_err(ERR_FILE_NOT_FOUND_OR_DENY)?;
            let p = fid_to_path(fid);
            tokio::fs::rename(&p, &p.with_extension("bak"))
                .await
                .unwrap();
            let mut file = File::create(&p).await.unwrap();
            let limit_by_ulv = ulv_trans_limit(ulv);
            if let Err(err) = write_body_to_file(body, &mut file, limit_by_ulv).await {
                db::data_d(fid_u64);
                file.set_len(0).await.ok();
                file.shutdown().await.ok(); // or flush?
                drop(file);
                tokio::fs::remove_file(&p).await.unwrap(); // ignore inner error?
                tokio::fs::rename(&p.with_extension("bak"), &p)
                    .await
                    .unwrap();
                return Err(err);
            }
            // TODO: use buffer len?
            db::data_u(fid_u64, file.metadata().await.unwrap().len(), meta);
            Ok([(TYPE_, HeaderValue::from_static(OK_DEFAULT))].into_response())
        }

        Op::Delete { token, fid } => {
            let (uid, _ulv) = token::vertify(token).cast_err(ERR_TOKEN)?;
            let fid_u64 = parse_slice::<u64>(fid).cast_err(ERR_HEADER_INVALID)?;
            let (owner_uid, _cap, _meta) =
                db::data_r(fid_u64).cast_err(ERR_FILE_NOT_FOUND_OR_DENY)?;
            (uid == owner_uid).cast_err(ERR_FILE_NOT_FOUND_OR_DENY)?;
            let p = fid_to_path(fid);
            db::data_d(fid_u64);
            tokio::fs::remove_file(p).await.unwrap();
            Ok([(TYPE_, HeaderValue::from_static(OK_DEFAULT))].into_response())
        }

        Op::Download {
            token,
            fid,
            limit,
            meta,
        } => {
            let (uid, ulv) = token::vertify(token).cast_err(ERR_TOKEN)?;
            let fid_u64 = parse_slice::<u64>(fid).cast_err(ERR_HEADER_INVALID)?;
            let (owner_uid, cap, meta) =
                db::data_r(fid_u64).cast_err(ERR_FILE_NOT_FOUND_OR_DENY)?;
            // allow all user to download any file?
            // (uid == owner_uid).cast_err(ERR_FILE_NOT_FOUND_OR_DENY)?;
            if cap <= ulv_trans_limit(ulv) as u64 {
                true
            } else {
                // if the file is big, query the owner's ulv
                let owner_ulv = db::user_r_ulv(uid).unwrap().0;
                cap <= ulv_share_limit(owner_ulv) as u64
            }
            .cast_err(ERR_SIZE_LIMIT)?;
            let limit = parse_slice::<u64>(limit).cast_err(ERR_HEADER_INVALID)?;
            let (mut response, len) = if cap <= limit {
                let file = File::open(fid_to_path(fid)).await.unwrap();
                (FileStream::new(file).into_response(), cap)
            } else {
                (().into_response(), 0)
            };
            let headers = response.headers_mut();
            headers.insert(CONTENT_LENGTH, HeaderValue::from(len));
            headers.insert(TYPE_, HeaderValue::from_static(OK_DEFAULT));
            headers.insert(META_, HeaderValue::from_buf(meta));
            Ok(response)
        }

        Op::List { token } => {
            let (uid, _ulv) = token::vertify(token).cast_err(ERR_TOKEN)?;
            let list = db::data_r_by_user(uid);
            let mut body = Vec::new(); // TODO: set capacity for performance
            for (fid, cap, mut meta) in list {
                write!(body, "fid:{fid}\nmeta:").unwrap();
                body.append(&mut meta);
                body.push(b'\n');
                body.push(b':');
                body.push(b'\n');
            }
            let mut response = body.into_response();
            let headers = response.headers_mut();
            headers.insert(TYPE_, HeaderValue::from_static(OK_DEFAULT));
            Ok(response)
        }
    }
}

pub fn service() -> Router {
    // TODO: vertify if the trigger fn not register!
    // TODO: user may make request immediately after the server launch, is this sound?
    // dbg!(STORAGE_ROOT.to_str());
    db::init();
    token::renew_tick();
    Router::new().route(
        "/paste",
        MethodRouter::new()
            .get(|| async {
                const PAGE: &str = (include_page!("page.html") as [_; 1])[0];
                const BODY: Html<Bytes> = Html(Bytes::from_static(PAGE.as_bytes()));
                // ([(CACHE_CONTROL, "max-age=600")], BODY)
                ([(CACHE_CONTROL, "no-store")], BODY)
            })
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

pub async fn dev() {}
