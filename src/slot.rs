use konst::for_range;
use konst::option::unwrap_or_else;
use konst::string::{find, split_at};

const fn slot_once(raw: &str) -> (&str, &str) {
    let mark = "/*{slot}*/";
    // #[rustc_const_unstable(feature = "const_option", issue = "67441")]
    let index = find(raw, mark, 0);
    let index = unwrap_or_else!(index, || panic!("slot mark not found"));
    let part_0 = split_at(raw, index).0;
    let part_1 = split_at(raw, index + mark.len()).1;
    (part_0, part_1)
}

/// Split template string by slot marks `/*{slot}*/`
///
/// # Example
///
/// ```
/// const RAW: &str = "<h1>/*{slot}*/</h1><p>/*{slot}*/</p>";
/// const PAGE: [&str; 3] = slot(RAW); // 2 slots split string into 3 parts
/// ```
///
/// # Panics
///
/// This function panics if `raw` doesn't have enough slot marks.
pub const fn slot<const N: usize>(raw: &str) -> [&str; N] {
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
    const RAW: &str = "<h1>/*{slot}*/</h1><p>/*{slot}*/</p>";
    const PAGE: [&str; 3] = slot(RAW);
    assert_eq!(PAGE, ["<h1>", "</h1><p>", "</p>"]);
}
