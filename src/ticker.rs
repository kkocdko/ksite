use std::iter::Iterator;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

const ANY: u64 = 99; // must >= 60, terser logic?

/// convert time stamp to (hours, minutes, seconds)
fn hms(v: u64) -> (u64, u64, u64) {
    (v / 60 / 60 % 24, v / 60 % 60, v % 60)
}

fn floor_by(v: u64, r: u64) -> u64 {
    v - v % r
}

fn gen_next(mut now: u64, cfg: (u64, u64, u64)) -> u64 {
    now += 1;
    let (ch, cm, cs) = cfg;
    loop {
        let (h, m, s) = hms(now);
        now = match (
            h == ch || ch == ANY,
            m == cm || cm == ANY,
            s == cs || cs == ANY,
        ) {
            // legal
            (true, true, true) => {
                return now;
            }

            // generate
            (true, true, _) if s < cs && s < 59 => {
                floor_by(now, 60) + if cs == ANY { s + 1 } else { cs }
            }
            (true, _, _) if m < cm && m < 59 => {
                floor_by(now, 60 * 60) + 60 * if cm == ANY { m + 1 } else { cm }
            }
            (_, _, _) if h < ch && h < 23 => {
                floor_by(now, 60 * 60 * 24) + 60 * 60 * if ch == ANY { h + 1 } else { ch }
            }

            // next day
            _ => floor_by(now, 60 * 60 * 24) + 60 * 60 * 24,
        };
    }
}

/// A `cron` like timed task util.
///
/// # Example
///
/// ```
/// let mut ticker = Ticker::new(&[(-1, 12, -1), (3, -1, 24)], 0);
/// loop {
///     // will be true if reached `XX:12:XX` or `03:XX:24`
///     dbg!(ticker.tick());
///     std::thread::sleep(std::time::Duration::from_millis(200));
/// }
/// ```
pub struct Ticker {
    next: AtomicU64,
    cfgs: Vec<(u64, u64, u64)>,
}

impl Ticker {
    /// Returns `true` if the next instant has been reached.
    pub fn tick(&self) -> bool {
        let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
        if now >= self.next.load(Ordering::SeqCst) {
            let nexts = self.cfgs.iter().map(|&cfg| gen_next(now, cfg));
            self.next.store(nexts.min().unwrap(), Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    /// Create `Ticker`.
    pub fn new(patterns: &[(i64, i64, i64)], zone: i64) -> Self {
        let mut cfgs = Vec::new();
        for &(h, m, s) in patterns {
            assert!(matches!((h, m, s), (-1..=23, -1..=59, -1..=59)));
            let h = if h == -1 { ANY } else { (h + 24 - zone) as _ } % 24;
            let m = if m == -1 { ANY } else { m as _ };
            let s = if s == -1 { ANY } else { s as _ };
            cfgs.push((h, m, s));
        }
        let ret = Ticker {
            next: AtomicU64::new(0),
            cfgs,
        };
        ret.tick();
        ret
    }

    /// Create with UTC+8 timezone.
    pub fn new_p8(patterns: &[(i64, i64, i64)]) -> Self {
        Self::new(patterns, 8)
    }
}
