use anyhow::Result;
use hyper::body::to_bytes;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Request};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use once_cell::sync::Lazy;
use std::time::SystemTime;
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

    let tls_config = ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let mut http_connector = HttpConnector::new();
    http_connector.enforce_http(false); // allow HTTPS
    let connector = HttpsConnectorBuilder::new()
        .with_tls_config(tls_config)
        .https_or_http() // allow both HTTPS and HTTP
        .enable_http1() // for a client, HTTP 1.1 is enough
        .wrap_connector(http_connector);

    Client::builder().build(connector)
});

/// Send the `Request` and returns response. Allow both HTTPS and HTTP
///
/// Unlike `reqwest` crate, this function dose not follow redirect
pub async fn fetch(request: Request<Body>) -> Result<Vec<u8>> {
    let response = CLIENT.request(request).await?;
    let bytes = to_bytes(response.into_body()).await?;
    Ok(bytes.into())
}

/// Fetch a URI, returns as text
pub async fn fetch_text(uri: &str) -> Result<String> {
    let uri = encode_uri(uri);
    let request = Request::get(uri).body(Body::empty())?;
    Ok(String::from_utf8(fetch(request).await?)?)
}

/// Fetch a URI which response json, get field by pointer
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
    Ok(v.trim_matches('"').to_string())
}

/// (stamp secs) -> (days)
pub fn elapse(stamp: f64) -> f64 {
    // javascript: new Date("2001.01.01 06:00").getTime()/1e3
    let now = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs_f64();
    (now - stamp) / 864e2 // unit: days
}

#[macro_export]
/// Care about the `Result`
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

/*
// #[rustc_const_unstable(feature = "const_option", issue = "67441")]
pub const fn expect_const<T: Copy>(i: Option<T>, tips: &str) -> T {
    match i {
        Some(v) => v,
        None => panic!("{}", tips),
    }
}

pub const fn unwrap_const<T: Copy>(i: Option<T>) -> T {
    expect_const(i, "Option is None")
}
*/

/// Split template page by slot marks `/*{slot}*/`
///
/// # Example
///
/// ```
/// // ./page.html:
/// "<h1>/*{slot}*/</h1><p>/*{slot}*/</p>";
/// // 2 slots split page into 3 parts
/// const PAGE: [&str; 3] = include_page!("page.html");
/// ```
#[macro_export]
macro_rules! include_page {
    ($file:expr) => {{
        include_page!(:include_str!($file))
    }};
    (:$raw:expr) => {{
        const __8: &str = $raw;
        // const __7: &str = const_str::replace!(__8, "\n        ", "\n");
        // const __6: &str = const_str::replace!(__7, "\n       ", "\n");
        // const __5: &str = const_str::replace!(__6, "\n      ", "\n");
        // const __4: &str = const_str::replace!(__5, "\n     ", "\n");
        // const __3: &str = const_str::replace!(__4, "\n    ", "\n");
        // const __2: &str = const_str::replace!(__3, "\n   ", "\n");
        // const __1: &str = const_str::replace!(__2, "\n  ", "\n");
        // const __0: &str = const_str::replace!(__1, "\n ", "\n");
        const_str::split!(__8, "/*{slot}*/")
    }};
}

const fn proc_page(raw: &str) -> &str {
    let n = 8;
    ""
}

#[test]
fn test_include_page() {
    const RAW: &str = "\
        <div>
            /*{slot}*/
            <p>Hi, /*{slot}*/</p>
        </div>
    ";
    const PAGE: [&str; 3] = include_page!(:RAW);
    assert_eq!(PAGE, ["<div>\n", "\n<p>Hi, ", "</p>\n</div>\n"]);
}

/*
pub const fn slot<const N: usize>(raw: &str) -> [&str; N] {
    let mark = b"/*{slot}*/
";
    // String::from_utf8_unchecked(bytes)
    // const_str::replace!()
    // const fn slot_once(raw: &str) -> (&str, &str) {
//     let mark = " /*{slot}*/
";
    //     let index = find(raw, mark, 0);
    //     let index = expect_const(index, "slot mark not found");
    //     let part_0 = split_at(raw, index).0;
    //     let part_1 = split_at(raw, index + mark.len()).1;
    //     (part_0, part_1)
    // }
    let mut p = raw;

    unsafe { std::str::from_utf8_unchecked(&[1, 2]) };
    let mut ret = [""; N];
    // #![feature(const_for)]
    // for_range! {i in 0..N - 2 =>
    //     (ret[i], p) = slot_once(p);
    // }
    // (ret[N - 2], ret[N - 1]) = slot_once(p);
    ret
}
*/
