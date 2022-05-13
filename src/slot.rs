use konst::option::unwrap_or_else;
use konst::slice::bytes_find;
use konst::slice::split_at;
use konst::string::from_utf8;
use konst::unwrap_ctx;

pub const fn slot(raw: &'static str) -> (&'static str, &'static str) {
    let raw = raw.as_bytes();
    let slot_mark = b"/*{slot}*/";
    let slot_index = bytes_find(raw, slot_mark, 0);
    let slot_index = unwrap_or_else!(slot_index, || panic!("slot mark not found"));
    let p0 = split_at(raw, slot_index).0;
    let p1 = split_at(raw, slot_index + slot_mark.len()).1;
    (unwrap_ctx!(from_utf8!(p0)), unwrap_ctx!(from_utf8!(p1)))
}
