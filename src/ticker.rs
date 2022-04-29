use std::iter::Iterator;
use std::time::SystemTime;

fn now_stamp() -> i64 {
    let epoch = SystemTime::UNIX_EPOCH;
    SystemTime::now().duration_since(epoch).unwrap().as_secs() as _
}
fn hms(v: i64) -> (i64, i64, i64) {
    (v / 3600 % 24, v / 60 % 60, v % 60)
}
fn floor_by(v: &mut i64, r: i64) {
    let t = *v % r;
    *v -= t;
}
fn gen_next(now: i64, cfg: (i64, i64, i64)) -> i64 {
    let mut stamp = now + 1;
    loop {
        let (h, m, s) = hms(stamp);
        let (ch, cm, cs) = cfg;
        match (
            h == ch || ch == ANY_VAL,
            m == cm || cm == ANY_VAL,
            s == cs || cs == ANY_VAL,
        ) {
            // legal
            (true, true, true) => {
                return stamp;
            }

            // generate
            (true, true, _) if s < cs && s < 59 => {
                floor_by(&mut stamp, 60);
                stamp += 1 * if cs == ANY_VAL { s + 1 } else { cs };
            }
            (true, _, _) if m < cm && m < 59 => {
                floor_by(&mut stamp, 60 * 60);
                stamp += 60 * if cm == ANY_VAL { m + 1 } else { cm };
            }
            (_, _, _) if h < ch && h < 23 => {
                floor_by(&mut stamp, 60 * 60 * 24);
                stamp += 60 * 60 * if ch == ANY_VAL { h + 1 } else { ch };
            }

            // next day
            _ => {
                floor_by(&mut stamp, 60 * 60 * 24);
                stamp += 60 * 60 * 24;
            }
        };
    }
}

const ANY_VAL: i64 = 99; // must >= 60, terser logic?

pub struct Ticker {
    next: i64,
    cfgs: Vec<(i64, i64, i64)>,
}

impl Ticker {
    pub fn tick(&mut self) -> bool {
        let now = now_stamp();
        if now >= self.next {
            let nexts = self.cfgs.iter().map(|cfg| gen_next(now, *cfg));
            self.next = nexts.min().unwrap();
            true
        } else {
            false
        }
    }
    fn new(patterns: impl Iterator<Item = (i64, i64, i64)>) -> Self {
        let mut cfgs = Vec::new();
        for (h, m, s) in patterns {
            assert!(h <= 24 && m <= 60 && s <= 60);
            assert!(h >= -1 && m >= -1 && s >= -1);
            let h = if h == -1 { ANY_VAL } else { h };
            let m = if m == -1 { ANY_VAL } else { m };
            let s = if s == -1 { ANY_VAL } else { s };
            cfgs.push((h, m, s));
        }
        let mut ret = Ticker { next: 0, cfgs };
        ret.tick();
        ret
    }
    pub fn new_utc(patterns: &[(i64, i64, i64)]) -> Self {
        const TIME_ZONE: i64 = 0;
        let transform = |&(h, m, s)| ((h + 24 - TIME_ZONE) % 24, m, s);
        Self::new(patterns.iter().map(transform))
    }
    pub fn new_p8(patterns: &[(i64, i64, i64)]) -> Self {
        const TIME_ZONE: i64 = 8;
        let transform = |&(h, m, s)| ((h + 24 - TIME_ZONE) % 24, m, s);
        Self::new(patterns.iter().map(transform))
    }
}
