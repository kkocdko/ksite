use anyhow::Result;
use std::time::SystemTime;

pub trait OptionResult<T> {
    fn e(self) -> Result<T>;
}

impl<T> OptionResult<T> for Option<T> {
    fn e(self) -> Result<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(anyhow::anyhow!("Option is None")),
        }
    }
}

/// Fetch a url, returns as text
pub async fn fetch_text(url: &str) -> Result<String> {
    Ok(reqwest::get(url).await?.text().await?)
}

/// Fetch a url which response json, get field by pointer
///
/// # Examples
///
/// ```
/// let result = await fetch_json("https://chrome.version.io", "/data/version"));
/// assert_eq!(result, Ok("1.2.0".to_string()));
/// ```
pub async fn fetch_json(url: &str, pointer: &str) -> Result<String> {
    let text = fetch_text(url).await?;
    let v = serde_json::from_str::<serde_json::Value>(&text)?;
    let v = v.pointer(pointer).e()?.to_string();
    Ok(v.trim_matches('"').to_string())
}

/// (epoch millis) -> (days)
pub fn elapse(time: f64) -> f64 {
    // javascript: new Date("2001.01.01 06:00").getTime()
    let epoch = SystemTime::UNIX_EPOCH;
    let now = SystemTime::now().duration_since(epoch).unwrap().as_millis() as f64;
    (now - time) / 864e5 // unit: days
}

#[macro_export]
/// Care about the Result
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

#[macro_export]
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

#[test]
fn test_include_page() {
    const RAW: &str = "\
        <div>
            /*{slot}*/
            <p>Hi, /*{slot}*/</p>
        </div>
    ";
    // const_str;
    const PAGE: [&str; 3] = include_page!(:RAW);
    assert_eq!(PAGE, ["<div>\n", "\n<p>Hi, ", "</p>\n</div>\n"]);
}

pub const fn slot<const N: usize>(raw: &str) -> [&str; N] {
    let mark = b"/*{slot}*/";
    // String::from_utf8_unchecked(bytes)
    // const_str::replace!()
    // const fn slot_once(raw: &str) -> (&str, &str) {
    //     let mark = "/*{slot}*/";
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
