use anyhow::Result;
use hyper::body::to_bytes;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Request};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
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
    const fn valid_table() -> [bool; VALIDS_LEN] {
        let mut table = [false; VALIDS_LEN];
        let valid_chars =
            b"!#$&'()*+,-./0123456789:;=?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]_abcdefghijklmnopqrstuvwxyz~";
        let mut i = 0;
        while i < valid_chars.len() {
            table[valid_chars[i] as usize] = true;
            i += 1;
        }
        table
    }

    const VALIDS_LEN: usize = u8::MAX as usize + 1;
    const VALIDS: [bool; VALIDS_LEN] = valid_table();

    fn hex(d: u8) -> u8 {
        match d {
            0..=9 => b'0' + d,
            10..=255 => b'A' - 10 + d,
        }
    }

    let mut o = Vec::with_capacity(i.len());
    for b in i.as_bytes() {
        if VALIDS[*b as usize] {
            o.push(*b);
        } else {
            o.push(b'%');
            o.push(hex(b >> 4));
            o.push(hex(b & 15));
        }
    }
    unsafe { String::from_utf8_unchecked(o) }
}

static CLIENT: Lazy<Client<HttpsConnector<HttpConnector>>> = Lazy::new(|| {
    // https://github.com/seanmonstar/reqwest/blob/v0.11.11/src/async_impl/client.rs#L340
    let trust_anchors = webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|trust_anchor| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            trust_anchor.subject,
            trust_anchor.spki,
            trust_anchor.name_constraints,
        )
    });
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(trust_anchors);

    let tls_cfg = ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let mut http_connector = HttpConnector::new();
    http_connector.enforce_http(false); // allow HTTPS
    let connector = HttpsConnectorBuilder::new()
        .with_tls_config(tls_cfg)
        .https_or_http() // allow both HTTPS and HTTP
        .enable_http1() // for a client, HTTP 1.1 is enough
        .wrap_connector(http_connector);

    Client::builder().build(connector)
});

/// Send the `Request` and returns response. Allow both HTTPS and HTTP.
///
/// Unlike `reqwest` crate, this function dose not follow redirect.
pub async fn fetch(request: Request<Body>) -> Result<Vec<u8>> {
    let response = CLIENT.request(request).await?;
    let bytes = to_bytes(response.into_body()).await?;
    Ok(bytes.into())
}

/// Fetch a URI, returns as text.
pub async fn fetch_text(uri: &str) -> Result<String> {
    let uri = encode_uri(uri);
    let request = Request::get(uri).body(Body::empty())?;
    Ok(String::from_utf8(fetch(request).await?)?)
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
pub async fn fetch_json(uri: &str, pointer: &str) -> Result<String> {
    let text = fetch_text(uri).await?;
    let v = serde_json::from_str::<serde_json::Value>(&text)?;
    let v = v.pointer(pointer).e()?.to_string();
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

// #[test]
// fn test_include_page() {
//     const RAW: &str = "\
//         <div>
//             /*{slot}*/
//             <p>Hi, /*{slot}*/</p>
//         </div>
//     ";
//     const PAGE: [&str; 3] = include_page!(:RAW);
//     assert_eq!(PAGE, ["<div>\n", "\n<p>Hi, ", "</p>\n</div>\n"]);
// }

#[test]
fn test_slot() {
    const RAW: &str = "<h1>/*{slot}*/</h1><p>/*{slot}*/</p>";
    // 2 slots split page into 3 parts
    const PAGE: [&str; 3] = slot(RAW);
    assert_eq!(PAGE, ["<h1>", "</h1><p>", "</p>"]);
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
