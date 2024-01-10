use anyhow::Result;
use axum::body::Body;
use axum::body::Bytes;
use axum::http::header::HOST;
use axum::http::Request;
use std::future::Future;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::UNIX_EPOCH;

/// While [`std::sync::LazyLock`](https://doc.rust-lang.org/stable/std/sync/struct.LazyLock.html) is still not in stable.
pub struct LazyLock<T> {
    f: fn() -> T,
    v: OnceLock<T>,
}

impl<T> LazyLock<T> {
    pub const fn new(f: fn() -> T) -> Self {
        Self {
            f,
            v: OnceLock::new(),
        }
    }
}

impl<T> std::ops::Deref for LazyLock<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.v.get_or_init(self.f)
    }
}

#[macro_export]
macro_rules! log {
    // TRAC | INFO | WARN | ERRO
    ($level:tt : $($arg:tt)*) => {{
        let stamp = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as u64;
        print!("[{stamp}:{}] ", stringify!($level));
        println!($($arg)*);
    }};
    ($($arg:tt)*) => {{
        $crate::log!(INFO : $($arg)*);
    }};
}

pub trait OptionResult<T> {
    fn e(self) -> Result<T>;
}

impl<T> OptionResult<T> for Option<T> {
    /// Convert `Option<T>` to `Result<T>`.
    fn e(self) -> Result<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(anyhow::anyhow!("Option is None")),
        }
    }
}

pub async fn with_retry<T, E, FUT>(
    f: impl Fn() -> FUT,
    limit: usize,
    interval_ms: u64,
) -> Result<T, E>
where
    E: std::fmt::Debug,
    FUT: Future<Output = Result<T, E>>,
{
    let mut err = None;
    for _ in 0..limit {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => err = Some(e),
        };
        tokio::time::sleep(Duration::from_millis(interval_ms)).await;
    }
    Err(err.unwrap())
}

/// Same as JavaScript's `encodeURI`.
pub fn encode_uri(i: &str) -> String {
    const fn gen_table() -> [bool; TABLE_LEN] {
        let mut table = [false; TABLE_LEN];
        let valid_chars =
            b"!#$&'()*+,-./0123456789:;=?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]_abcdefghijklmnopqrstuvwxyz~";
        let mut i = 0;
        while i < valid_chars.len() {
            table[valid_chars[i] as usize] = true;
            i += 1;
        }
        table
    }

    const TABLE_LEN: usize = u8::MAX as usize + 1; // == 256
    const IS_VALID: [bool; TABLE_LEN] = gen_table();

    fn to_hex(d: u8) -> u8 {
        match d {
            0..=9 => d + b'0',
            10..=255 => d - 10 + b'a', // regardless of upper or lower case
        }
    }

    let mut o = Vec::with_capacity(i.len());
    for b in i.as_bytes() {
        if IS_VALID[*b as usize] {
            o.push(*b);
        } else {
            o.push(b'%');
            o.push(to_hex(b >> 4));
            o.push(to_hex(b & 15));
        }
    }
    unsafe { String::from_utf8_unchecked(o) }
}

/// Escape log string into a single line, html safe string.
///
/// `[a"foo\nbar]` into `[a&quot;foo\\nbar]`
pub fn log_escape(s: &str) -> String {
    html_escape(&s.replace('\n', "\\n"))
}

/// Escape to HTML safe string.
pub fn html_escape(v: &str) -> String {
    askama_escape::escape(v, askama_escape::Html).to_string()
}

pub fn str2req(s: impl AsRef<str>) -> Request<Body> {
    let uri = encode_uri(s.as_ref());
    let uri = axum::http::Uri::try_from(uri).unwrap();
    Request::get(&uri)
        .header(HOST, uri.host().unwrap())
        .body(Body::empty())
        .unwrap()
}

// The HTTP/HTTPS client.
pub static CLIENT: LazyLock<tls_http::Client> =
    LazyLock::new(tls_http::Client::new_with_webpki_roots);

/// Fetch a URI, returns as `Vec<u8>`.
pub async fn fetch_data(req: Request<Body>) -> Result<Bytes> {
    let res = CLIENT.fetch(req, None).await?;
    let body = res.into_body();
    Ok(axum::body::to_bytes(axum::body::Body::new(body), usize::MAX).await?)
}

/// Fetch a URI, returns as `String`.
pub async fn fetch_text(request: Request<Body>) -> Result<String> {
    let body = fetch_data(request).await?;
    Ok(String::from_utf8(Vec::from(body))?)
}

/// Fetch a URI which response json, get field by pointer. Value will be convert to string always.
///
/// # Examples
///
/// ```
/// let v = await fetch_json("https://example.com", "/data/size"));
/// // is this? { "data": { "size": "1024" } }
/// // or this? { "data": { "size": 1024 } }
/// assert_eq!(v, Ok("1024".to_string())); // the same result!
/// ```
pub async fn fetch_json(request: Request<Body>, pointer: &str) -> Result<String> {
    let text = fetch_text(request).await?;
    let v = serde_json::from_str::<serde_json::Value>(&text)?;
    let v = v
        .pointer(pointer)
        .ok_or_else(|| anyhow::anyhow!("json field not found"))?
        .to_string();
    Ok(v.trim_matches('"').to_owned())
}

/// (stamp secs) -> (days)
pub fn elapse(stamp: f64) -> f64 {
    // javascript: new Date("2001.01.01 06:00").getTime()/1e3
    let now = UNIX_EPOCH.elapsed().unwrap().as_secs_f64();
    (now - stamp) / 864e2 // unit: days
}

#[macro_export]
/// Care about the `Result`.
macro_rules! care {
    ($result:expr) => {{
        let result = $result;
        if let Err(e) = &result {
            $crate::log!(ERRO : "[cared error] {}:{} {:?}", file!(), line!(), e);
        }
        result
    }};
    ($result:expr, $if_err:tt) => {{
        match care!($result) {
            Ok(v) => v,
            _ => $if_err,
        }
    }};
}

/// # Use macros instead of call inner functions directly!
///
/// Operations about const string.
///
pub mod str_const_ops_ {
    const fn kmp<const A: usize, const B: usize>(s: &[u8], p: [u8; A]) -> [usize; B] {
        let mut next = [0; A];
        let mut i = 1;
        let mut j = 0;
        while i < A {
            while j != 0 && p[i] != p[j] {
                j = next[j - 1];
            }
            if p[i] == p[j] {
                j += 1;
            }
            next[i] = j;
            i += 1;
        }

        let mut ret = [usize::MAX; B];
        let mut i = 0;
        let mut j = 0;
        let mut k = 0;
        while i < s.len() {
            while j != 0 && s[i] != p[j] {
                j = next[j - 1];
            }
            if s[i] == p[j] {
                j += 1;
            }
            if j == A {
                ret[k] = i - A + 1;
                k += 1;
                j = next[j - 1];
            }
            i += 1;
        }

        // TODO: add err info here
        // if k != B {
        //     panic!();
        // }

        ret
    }

    /// Split template page by slot marks `/*{slot}*/`.
    ///
    /// # Example
    ///
    /// ```
    /// const RAW: &str = "<h1>/*{slot}*/</h1><p>/*{slot}*/</p>";
    /// // 2 slots split page into 3 parts
    /// const PAGE: [&str; 3] = slot(RAW);
    /// assert_eq!(PAGE, ["<h1>", "</h1><p>", "</p>"]);
    /// ```
    pub const fn slot<const N: usize>(raw: &str) -> [&str; N] {
        if N == 1 {
            let mut ret = [""; N];
            ret[0] = raw;
            return ret;
        }
        const MARK: [u8; 10] = *b"/*{slot}*/";
        let raw = raw.as_bytes();
        let idxs: [usize; N] = kmp(raw, MARK);
        // let idxs = unwrap_o(idxs.as_slice().split_last()).1; // real len is n-1;

        let mut ret_b = [b"".as_slice(); N];
        let mut i = 0;
        while i < N {
            let (begin, end) = if i == 0 {
                (0, idxs[i])
            } else if i != N - 1 {
                (idxs[i - 1] + MARK.len(), idxs[i])
            } else {
                (idxs[i - 1] + MARK.len(), raw.len())
            };

            // ret_b[i] = &raw[begin..end]; // is unusable in const fn
            ret_b[i] = unsafe { std::slice::from_raw_parts(raw.as_ptr().add(begin), end - begin) };
            i += 1;
        }

        let mut ret = [""; N];
        let mut i = 0;
        while i < N {
            ret[i] = unsafe {
                // this's safe certainly, we don't touch any part of str, and the
                // split edge is `MARK` which only includes ASCII chars
                std::str::from_utf8_unchecked(ret_b[i])
            };
            i += 1;
        }
        ret
    }

    pub const fn strip_get_len(src: &[u8]) -> usize {
        // lite version of strip_do
        let mut buf_i = 0;
        let mut src_i = 0;
        loop {
            while src_i < src.len() && (src[src_i] == b'\n' || src[src_i] == b' ') {
                src_i += 1;
            }
            while src_i < src.len() && src[src_i] != b'\n' {
                // buf[buf_i] = src[src_i];
                buf_i += 1;
                src_i += 1;
            }
            if src_i < src.len() {
                // buf[buf_i] = b'\n';
                buf_i += 1;
            } else {
                break;
            }
        }
        buf_i
    }

    pub const fn strip_do<const LEN: usize>(src: &[u8]) -> [u8; LEN] {
        let mut buf: [u8; LEN] = [0; LEN];
        let mut buf_i = 0;
        let mut src_i = 0;
        loop {
            while src_i < src.len() && (src[src_i] == b'\n' || src[src_i] == b' ') {
                src_i += 1;
            }
            while src_i < src.len() && src[src_i] != b'\n' {
                buf[buf_i] = src[src_i];
                buf_i += 1;
                src_i += 1;
            }
            if src_i < src.len() {
                buf[buf_i] = b'\n';
                buf_i += 1;
            } else {
                break;
            }
        }
        buf
    }
}

#[macro_export]
macro_rules! strip_str {
    ($s:expr) => {{
        #[cfg(debug_assertions)]
        {
            $s
        }
        #[cfg(not(debug_assertions))]
        {
            use $crate::utils::str_const_ops_::*;
            // thanks: https://docs.rs/const-str/0.4.3/const_str/macro.replace.html
            const RAW: &[u8] = $s.as_bytes();
            const BUF: [u8; strip_get_len(RAW)] = strip_do(RAW);
            const RET: &str = unsafe { std::str::from_utf8_unchecked(&BUF) };
            RET
        }
    }};
}

/// Include a source code file, with solt detect and blank strip.
#[macro_export]
macro_rules! include_src {
    ($s:expr) => {{
        use $crate::utils::str_const_ops_::*;
        const S: &str = $crate::strip_str!(include_str!($s));
        slot(S)
    }};
}

#[allow(unused)]
pub fn _test_include_page() {
    use str_const_ops_::*;
    const RAW: &str = "
        <div>
            /*{slot}*/
            <p>Hi, /*{slot}*/</p>
        </div>
    ";
    const PAGE: [&str; 3] = {
        const S: &[u8] = RAW.as_bytes();
        const BUF: [u8; strip_get_len(S)] = strip_do(S);
        slot(unsafe { std::str::from_utf8_unchecked(&BUF) })
    };
    assert_eq!(PAGE, ["\n<div>\n", "\n<p>Hi, ", "</p>\n</div>\n"]);
}

/// Detect the `strip_str` works or not.
#[allow(unused)]
pub fn _detect_str_in_binary() {
    let s = std::fs::read("ksite").unwrap();
    let p = b"DELETE FROM health_log";
    for i in 0..s.len() {
        let mut m = true;
        for j in 0..p.len() {
            if i + j >= s.len() || s[i + j] != p[j] {
                m = false;
                break;
            }
        }
        if m {
            let i1 = (i - 8).clamp(0, s.len() - 1);
            let i2 = (i + 64).clamp(0, s.len() - 1);
            log!(">>> {:?}", String::from_utf8_lossy(&s[i1..=i2]));
        }
    }
}

// MethodRouter::new().get(
//     |u: WebSocketUpgrade, c: ConnectInfo<SocketAddr>| async move {
//         if c.0.ip() != IpAddr::V4(Ipv4Addr::LOCALHOST) {
//             return "only allowed for localhost".into_response();
//         }
//         u.on_upgrade(ws_handler)
//     },
// )
