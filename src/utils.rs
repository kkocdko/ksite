use anyhow::Result;
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};
use hyper_rustls::HttpsConnector;
use once_cell::sync::Lazy;
use std::time::UNIX_EPOCH;
use tokio_rustls::rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore};

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

    const TABLE_LEN: usize = u8::MAX as usize + 1; // 256
    const IS_VALID: [bool; TABLE_LEN] = gen_table();

    fn hex(d: u8) -> u8 {
        match d {
            0..=9 => b'0' + d,
            10..=255 => b'A' - 10 + d,
        }
    }

    let mut o = Vec::with_capacity(i.len());
    for b in i.as_bytes() {
        if IS_VALID[*b as usize] {
            o.push(*b);
        } else {
            o.push(b'%');
            o.push(hex(b >> 4));
            o.push(hex(b & 15));
        }
    }
    unsafe { String::from_utf8_unchecked(o) }
}

/// Read `hyper::Body` into `Vec<u8>`, returns emply if reached the limit size (2 MiB).
///
/// Simpler than `hyper::body::to_bytes`.
pub async fn read_body(mut body: Body) -> Vec<u8> {
    let mut v = Vec::new();
    while let Some(Ok(bytes)) = body.data().await {
        v.append(&mut bytes.into());
        // 2 MiB
        if v.len() > 2048 * 1024 {
            v.clear();
            break;
        }
    }
    v
}

static CLIENT: Lazy<Client<HttpsConnector<HttpConnector>>> = Lazy::new(|| {
    // https://github.com/seanmonstar/reqwest/blob/v0.11.11/src/async_impl/client.rs#L340
    let root_cert_store = RootCertStore {
        roots: { webpki_roots::TLS_SERVER_ROOTS.0.iter() }
            .map(|trust_anchor| {
                OwnedTrustAnchor::from_subject_spki_name_constraints(
                    trust_anchor.subject,
                    trust_anchor.spki,
                    trust_anchor.name_constraints,
                )
            })
            .collect(),
    };
    let tls_cfg = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    // tls_cfg.alpn_protocols = vec![b"http/1.1".to_vec()]; // http2 is not supported
    let mut http_conn = HttpConnector::new();
    http_conn.enforce_http(false); // allow HTTPS
    let connector = HttpsConnector::from((http_conn, tls_cfg));
    Client::builder().build(connector)
});

pub trait IntoRequest {
    fn into_request(self) -> Request<Body>;
}
impl IntoRequest for Request<Body> {
    fn into_request(self) -> Request<Body> {
        self
    }
}
impl IntoRequest for &str {
    fn into_request(self) -> Request<Body> {
        let ret = Request::get(encode_uri(self)).body(Body::empty()).unwrap();
        ret.into_request()
    }
}
impl IntoRequest for &String {
    fn into_request(self) -> Request<Body> {
        self.as_str().into_request()
    }
}

/// Send a `Request` and return the response. Allow both HTTPS and HTTP.
///
/// Unlike `reqwest` crate, this function dose not follow redirect.
pub async fn fetch(request: impl IntoRequest) -> Result<Response<Body>> {
    Ok(CLIENT.request(request.into_request()).await?)
}

/// Fetch a URI, returns as text.
pub async fn fetch_text(request: impl IntoRequest) -> Result<String> {
    let response = fetch(request).await?;
    let body = read_body(response.into_body()).await;
    Ok(String::from_utf8(body)?)
}

/// Fetch a URI which response json, get field by pointer.
///
/// # Examples
///
/// ```
/// // value will be convert to string, the field type is not cared
/// let v = await fetch_json("https://api.io", "/data/size"));
/// // is this? { "data": { "size": "1024" } }
/// // or this? { "data": { "size": 1024 } }
/// assert_eq!(v, Ok("1024".to_string())); // the same result!
/// ```
pub async fn fetch_json(request: impl IntoRequest, pointer: &str) -> Result<String> {
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
            eprintln!("[cared error] {}:{} {:?}", file!(), line!(), e);
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
                core::str::from_utf8_unchecked(ret_b[i])
            };
            i += 1;
        }
        ret
    }

    pub const fn strip_get_len(s: &[u8]) -> usize {
        let mut len = 0;
        let mut idx = 0;
        while idx < s.len() {
            if s[idx] == b'\n' {
                idx += 1;
                len += 1;
                while idx < s.len() && s[idx] == b' ' {
                    idx += 1;
                }
            } else {
                idx += 1;
                len += 1;
            }
        }
        len
    }

    pub const fn strip_do<const LEN: usize>(src: &[u8]) -> [u8; LEN] {
        let mut buf: [u8; LEN] = [0; LEN];
        let mut buf_i = 0;
        let mut src_i = 0;
        while src_i < src.len() {
            buf[buf_i] = src[src_i];
            buf_i += 1;
            src_i += 1;
            if src[src_i - 1] == b'\n' {
                while src_i < src.len() && src[src_i] == b' ' {
                    src_i += 1;
                }
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
            const S: &[u8] = $s.as_bytes();
            const BUF: [u8; strip_get_len(S)] = strip_do(S);
            unsafe { std::str::from_utf8_unchecked(&BUF) }
        }
    }};
}

#[macro_export]
macro_rules! include_page {
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
            println!(">>> {:?}", String::from_utf8_lossy(&s[i1..=i2]));
        }
    }
}
