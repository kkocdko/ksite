use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::{Duration, UNIX_EPOCH};

// Tue, 05 Sep 2023 15:05:25 GMT

const ANY: u64 = 99; // must >= 60, terser logic?

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
        // log!("now = {now}, next = {next}, ret = {ret}");
        if now >= next {
            let nexts = self.cfgs.iter().map(|&cfg| gen_next(now, cfg));
            self.next.store(nexts.min().unwrap(), Ordering::SeqCst);
        }
        ret
    }
}

#[macro_export]
macro_rules! ticker {
    ($zone:literal, $($pattern:expr),*) => {
        use $crate::ticker::*;
        use std::sync::atomic::AtomicU64;
        const N: usize = [$($pattern),*].len();
        static CFGS: [(u64, u64, u64); N] = parse_patterns($zone, [$($pattern),*]);
        static TICKER: Ticker = Ticker {
            next: AtomicU64::new(0),
            cfgs: &CFGS,
        };
        if !TICKER.tick() {
            return;
        }
    };
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
            ($pidx:expr,$vidx:tt) => {
                if p[$pidx] == b'X' {
                    assert!(p[$pidx + 1] == b'X');
                    cfgs[i].$vidx = ANY;
                } else {
                    assert!(b'0' <= p[$pidx] && p[$pidx] < b'9');
                    assert!(b'0' <= p[$pidx + 1] && p[$pidx + 1] <= b'9');
                    cfgs[i].$vidx += (p[$pidx] - b'0') as u64 * 10;
                    cfgs[i].$vidx += (p[$pidx + 1] - b'0') as u64;
                }
            };
        }
        part_code!(0, 0);
        part_code!(3, 1);
        part_code!(6, 2);
        if cfgs[i].0 != ANY {
            cfgs[i].0 = ((cfgs[i].0 as i64 + 24 - zone) % 24) as u64;
        }
        i += 1;
    }
    cfgs
}

fn get_now() -> u64 {
    UNIX_EPOCH.elapsed().unwrap().as_secs() as _
}

// bugtick: Tue, 28 Feb 2023 03:27:50 GMT
// bugtick: Thu, 09 Mar 2023 16:04:29 GMT

// TODO add fuzzle test
#[allow(unused)]
fn get_now_fake() -> i64 {
    use crate::utils::LazyLock;

    static V: LazyLock<AtomicI64> = LazyLock::new(|| {
        AtomicI64::new(
            httpdate::parse_http_date("Tue, 28 Feb 2023 02:27:50 GMT")
                .unwrap()
                .elapsed()
                .unwrap()
                .as_secs() as _,
        )
    });
    print!("now = {} ", {
        let t = V.load(Ordering::SeqCst);
        httpdate::fmt_http_date(UNIX_EPOCH + Duration::from_secs(t as _))
    });
    V.fetch_add(1, Ordering::SeqCst)
}
#[allow(unused)]
mod test_helper {
    use std::time::{Duration, UNIX_EPOCH};

    pub fn date2stamp(s: &str) -> u64 {
        let t = httpdate::parse_http_date(s).unwrap();
        t.duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
    pub fn stamp2date(t: u64) -> String {
        httpdate::fmt_http_date(UNIX_EPOCH + Duration::from_secs(t as _))
    }
}

#[allow(unused)]
pub async fn fuzzle_test() {
    // let c = (3, ANY, 24);
    // let mut now = test_helper::date2stamp("Mon, 30 Jan 2023 02:27:50 GMT") as i64;
    // let mut end = test_helper::date2stamp("Tue, 28 Feb 2023 02:27:50 GMT") as i64;
    // loop {
    //     let next = gen_next(now, c);
    //     let next_stupid = gen_next_stupid(now, c);
    //     if next != next_stupid {
    //         let v1 = test_helper::stamp2date(next as _);
    //         let v2 = test_helper::stamp2date(next_stupid as _);
    //         log!("should be {v2} , not {v1}")
    //     }
    //     now = next;
    //     // let v1 = test_helper::stamp2httpdate(next as _);
    //     // log!("should be {v1}");
    //     if now >= end {
    //         log!("end");
    //         std::process::exit(0);
    //     }
    // }

    // https://crates.io/crates/cron
    // use std::time::Duration;
    // let interval = Duration::from_millis(50);
    // // let interval = Duration::from_secs(1);
    // log!("oscillator interval = {interval:?}");
    // let mut interval = tokio::time::interval(interval);
    // let mut ticker = Ticker::new(&[(-1, 4, 0)], 0);
    // // let mut ticker = Ticker::new(&[(-1, 12, -1), (3, -1, 24)], 0);
    // // static TICKER: Lazy<Ticker> = Lazy::new(|| Ticker::new_p8(&[(-1, 4, 0)]));
    // loop {
    //     interval.tick().await;
    //     if ticker.tick() {
    //         log!("tick");
    //     } else {
    //         log!("no-tick");
    //     }
    //     // let _ = tokio::join!(
    //     //     units::magazine::tick(),
    //     //     units::qqbot::tick(),
    //     // );
    // }
}
