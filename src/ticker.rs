use std::sync::atomic::{AtomicU64, Ordering};

fn gen_next(mut now: u64, cfg: (u64, u64, u64)) -> u64 {
    /// Convert time stamp to (hours, minutes, seconds).
    fn hms(v: u64) -> (u64, u64, u64) {
        (v / 60 / 60 % 24, v / 60 % 60, v % 60)
    }
    fn floor_by(v: u64, r: u64) -> u64 {
        v - v % r
    }
    now += 1;
    let (ch, cm, cs) = cfg;
    loop {
        let (h, m, s) = hms(now);
        now = match (
            h == ch || ch == u64::MAX,
            m == cm || cm == u64::MAX,
            s == cs || cs == u64::MAX,
        ) {
            // legal
            (true, true, true) => {
                return now;
            }

            // generate
            (true, true, _) if s < cs && s < 59 => {
                floor_by(now, 60) + if cs == u64::MAX { s + 1 } else { cs }
            }
            (true, _, _) if m < cm && m < 59 => {
                floor_by(now, 60 * 60) + 60 * if cm == u64::MAX { m + 1 } else { cm }
            }
            (_, _, _) if h < ch && h < 23 => {
                floor_by(now, 60 * 60 * 24) + 60 * 60 * if ch == u64::MAX { h + 1 } else { ch }
            }

            // next day
            _ => floor_by(now, 60 * 60 * 24) + 60 * 60 * 24,
        };
    }
}

/// A `cron` like timed task util.
pub struct Ticker {
    pub next: AtomicU64,
    pub cfgs: &'static [(u64, u64, u64)],
}

impl Ticker {
    /// Returns `true` if the next instant has been reached.
    pub fn tick(&self) -> bool {
        let now = get_now();
        let next = self.next.load(Ordering::SeqCst);
        let ret = now >= next && next != 0;
        if now >= next {
            let nexts = self.cfgs.iter().map(|&cfg| gen_next(now, cfg));
            self.next.store(nexts.min().unwrap(), Ordering::SeqCst);
        }
        ret
    }
}

#[macro_export]
macro_rules! ticker {
    ($if_not_tick:tt, $zone:literal, $($pattern:expr),*) => {{
        use $crate::ticker::*;
        use std::sync::atomic::AtomicU64;
        const N: usize = [$($pattern),*].len();
        static CFGS: [(u64, u64, u64); N] = parse_patterns($zone, [$($pattern),*]);
        static TICKER: Ticker = Ticker {
            next: AtomicU64::new(0),
            cfgs: &CFGS,
        };
        if !TICKER.tick() {
            $if_not_tick;
        }
    }};
}

pub const fn parse_patterns<const N: usize>(
    zone: i64,
    patterns: [&'static str; N],
) -> [(u64, u64, u64); N] {
    let mut cfgs: [(u64, u64, u64); N] = [(0, 0, 0); N];
    let mut i = 0;
    while i < N {
        let p = patterns[i].as_bytes();
        macro_rules! part_code {
            ($pidx:expr, $vidx:tt) => {
                if p[$pidx] == b'X' {
                    assert!(p[$pidx + 1] == b'X');
                    cfgs[i].$vidx = u64::MAX;
                } else {
                    assert!(matches!(p[$pidx], b'0'..=b'9'));
                    assert!(matches!(p[$pidx + 1], b'0'..=b'9'));
                    cfgs[i].$vidx = (p[$pidx] - b'0') as u64 * 10 + (p[$pidx + 1] - b'0') as u64;
                }
            };
        }
        part_code!(0, 0);
        part_code!(3, 1);
        part_code!(6, 2);
        if cfgs[i].0 != u64::MAX {
            cfgs[i].0 = ((cfgs[i].0 as i64 + 24 - zone) % 24) as u64;
        }
        i += 1;
    }
    cfgs
}

fn get_now() -> u64 {
    std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as _
}

// fn get_now() -> u64 {
//     static CUR: AtomicU64 = AtomicU64::new(3000000000000);
//     CUR.fetch_add(1, Ordering::SeqCst)
// }

// pub fn fuzzle_test() {
// ticker!()
// }

// bugtick: Tue, 28 Feb 2023 03:27:50 GMT
// bugtick: Thu, 09 Mar 2023 16:04:29 GMT
// bugtick: Tue, 05 Sep 2023 15:05:25 GMT
