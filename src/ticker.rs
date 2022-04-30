use std::iter::Iterator;
use std::time::SystemTime;

const ANY: i64 = 99; // must >= 60, terser logic?

fn now_stamp() -> i64 {
    let epoch = SystemTime::UNIX_EPOCH;
    SystemTime::now().duration_since(epoch).unwrap().as_secs() as _
}
fn hms(v: i64) -> (i64, i64, i64) {
    (v / 60 / 60 % 24, v / 60 % 60, v % 60)
}
fn floor_by(v: i64, r: i64) -> i64 {
    v - v % r
}
fn gen_next(now: i64, cfg: (i64, i64, i64)) -> i64 {
    let mut stamp = now + 1;
    let (ch, cm, cs) = cfg;
    loop {
        let (h, m, s) = hms(stamp);
        stamp = match (
            h == ch || ch == ANY,
            m == cm || cm == ANY,
            s == cs || cs == ANY,
        ) {
            // legal
            (true, true, true) => {
                return stamp;
            }

            // generate
            (true, true, _) if s < cs && s < 59 => {
                floor_by(stamp, 60) + if cs == ANY { s + 1 } else { cs }
            }
            (true, _, _) if m < cm && m < 59 => {
                floor_by(stamp, 60 * 60) + 60 * if cm == ANY { m + 1 } else { cm }
            }
            (_, _, _) if h < ch && h < 23 => {
                floor_by(stamp, 60 * 60 * 24) + 60 * 60 * if ch == ANY { h + 1 } else { ch }
            }

            // next day
            _ => floor_by(stamp, 60 * 60 * 24) + 60 * 60 * 24,
        };
    }
}

/// A `cron` like timed task util.
///
/// # Example
///
/// ```
/// // At any hour's 12 minute's every seconds, `ticker.tick()` will be true once.
/// let mut ticker = Ticker::new(&[(-1, 12, -1)], 0);
/// loop {
///     println!("{:?}", ticker.tick());
///     std::thread::sleep(std::time::Duration::from_millis(200));
/// }
/// ```
pub struct Ticker {
    next: i64,
    cfgs: Vec<(i64, i64, i64)>,
}

impl Ticker {
    pub fn tick(&mut self) -> bool {
        let now = now_stamp();
        if now >= self.next {
            let nexts = self.cfgs.iter().map(|&cfg| gen_next(now, cfg));
            self.next = nexts.min().unwrap();
            true
        } else {
            false
        }
    }
    pub fn new(patterns: &[(i64, i64, i64)], zone: i64) -> Self {
        let mut cfgs = Vec::new();
        for &(h, m, s) in patterns {
            assert!(h <= 23 && m <= 59 && s <= 59);
            assert!(h >= -1 && m >= -1 && s >= -1);
            let h = if h == -1 { ANY } else { (h + 24 - zone) % 24 };
            let m = if m == -1 { ANY } else { m };
            let s = if s == -1 { ANY } else { s };
            cfgs.push((h, m, s));
        }
        let mut ret = Ticker { next: 0, cfgs };
        ret.tick();
        ret
    }
    pub fn new_p8(patterns: &[(i64, i64, i64)]) -> Self {
        Self::new(patterns, 8)
    }
}
