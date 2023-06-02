use std::sync::atomic::{AtomicI64, Ordering};
use std::time::UNIX_EPOCH;

const ANY: i64 = 99; // must >= 60, terser logic?

fn get_now() -> i64 {
    // return get_now_fake();
    // https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#semantics
    // > Casting between two integers of the same size (e.g. i32 -> u32) is a no-op
    UNIX_EPOCH.elapsed().unwrap().as_secs() as _
}

/// Convert time stamp to (hours, minutes, seconds).
fn hms(v: i64) -> (i64, i64, i64) {
    (v / 60 / 60 % 24, v / 60 % 60, v % 60)
}

fn floor_by(v: i64, r: i64) -> i64 {
    v - v % r
}

fn gen_next(mut now: i64, cfg: (i64, i64, i64)) -> i64 {
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
    next: AtomicI64,
    cfgs: Vec<(i64, i64, i64)>,
}

impl Ticker {
    /// Returns `true` if the next instant has been reached.
    pub fn tick(&self) -> bool {
        let now = get_now();
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
            let h = if h == -1 { ANY } else { (h + 24 - zone) % 24 };
            let m = if m == -1 { ANY } else { m };
            let s = if s == -1 { ANY } else { s };
            cfgs.push((h, m, s));
        }
        let ret = Ticker {
            next: AtomicI64::new(0),
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

// TODO add fuzzle test
#[allow(unused)]
fn get_now_fake() -> i64 {
    static V: AtomicI64 = AtomicI64::new(0);
    println!("now = {}", V.load(Ordering::SeqCst));
    V.fetch_add(1, Ordering::SeqCst)
}
#[allow(unused)]
pub async fn fuzzle_test() {
    use std::time::Duration;
    let interval = Duration::from_secs(1);
    println!("oscillator interval = {interval:?}");
    let mut interval = tokio::time::interval(interval);
    let mut ticker = Ticker::new(&[(-1, 12, -1), (3, -1, 24)], 0);
    loop {
        interval.tick().await;
        println!("tick");
        // let _ = tokio::join!(
        //     units::magazine::tick(),
        //     units::qqbot::tick(),
        // );
    }
}
