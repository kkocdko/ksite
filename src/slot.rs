use konst::for_range;
use konst::option::unwrap_or_else;
use konst::string::{find, split_at};

const fn slot_once(raw: &'static str) -> (&'static str, &'static str) {
    let slot_mark = "/*{slot}*/";
    // #[rustc_const_unstable(feature = "const_option", issue = "67441")]
    let slot_index = find(raw, slot_mark, 0);
    let slot_index = unwrap_or_else!(slot_index, || panic!("slot mark not found"));
    let p0 = split_at(raw, slot_index).0;
    let p1 = split_at(raw, slot_index + slot_mark.len()).1;
    (p0, p1)
}

/// Split template string by slot marks `/*{slot}*/`
///
/// # Example
///
/// ```
/// const RAW: &str = r#"<h1>/*{slot}*/</h1><p>/*{slot}*/</p>"#;
/// const PAGE: [&str; 3] = slot(RAW); // 2 slots split string into 3 parts
/// ```
///
/// # Panics
///
/// This function panics if `raw` doesn't have enough slot marks.
///
pub const fn slot<const N: usize>(raw: &'static str) -> [&'static str; N] {
    let mut p = raw;
    let mut ret = [""; N];
    // #![feature(const_for)]
    for_range! {i in 0..N - 2 =>
        (ret[i], p) = slot_once(p);
    }
    (ret[N - 2], ret[N - 1]) = slot_once(p);
    ret
}

#[test]
fn test() {
    const RAW: &str = r#"<h1>/*{slot}*/</h1><p>/*{slot}*/</p>"#;
    const PAGE: [&str; 3] = slot(RAW);
    assert_eq!(PAGE, ["<h1>", "</h1><p>", "</p>"]);
}
