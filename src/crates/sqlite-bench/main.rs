//！ benchmarks

const SCALE: u64 = 1000;

// SELECT * FROM sqlite_master WHERE type = 'index';

use std::sync::OnceLock;

use rusqlite::Connection;

pub struct LazyLock<T> {
    f: fn() -> T,
    v: OnceLock<T>,
}

impl<T> LazyLock<T> {
    pub const fn new(f: fn() -> T) -> Self {
        Self {
            f,
            v: OnceLock::new(),
        }
    }
}

impl<T> std::ops::Deref for LazyLock<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.v.get_or_init(self.f)
    }
}

pub mod mono {
    use std::sync::{Arc, Mutex};

    pub struct Mono<I>(Mutex<I>);
    impl<I: Send + 'static> Mono<I> {
        pub fn new(init: I) -> Self {
            Self(Mutex::new(init))
        }
        pub async fn call<T: Send + 'static>(
            &'static self,
            f: impl Fn(&mut I) -> T + Send + 'static,
        ) -> T {
            let inner = &self.0;
            tokio::task::spawn_blocking(move || {
                let mut inner = inner.lock().unwrap();
                let inner = &mut *inner;
                f(inner)
            })
            .await
            .unwrap()
        }
    }

    use tokio::sync::mpsc;
    pub struct MonoM<T> {
        tx: mpsc::Sender<Box<dyn FnOnce(&mut T) -> () + Send>>,
    }
    impl<T: Send + 'static> MonoM<T> {
        pub fn new(mut v: T) -> Self {
            let (tx, mut rx) = mpsc::channel::<Box<dyn FnOnce(&mut T) -> () + Send>>(1);
            std::thread::spawn(move || {
                // after self.tx drop, the recv() here will cause thread exit, without memory leaking
                while let Some(f) = rx.blocking_recv() {
                    f(&mut v);
                }
            });
            Self { tx }
        }
        pub async fn call<R: Send + 'static>(&self, f: impl Fn(&mut T) -> R + Send + 'static) -> R {
            let mut res = Arc::new(tokio::sync::Mutex::const_new(None));
            let mut guard = res.clone().lock_owned().await;
            self.tx
                .send(Box::new(move |s| *guard = Some(f(s)))) // f may be inlined, it's fine
                .await
                .unwrap();
            let mut g = res.lock().await;
            g.take().unwrap()
            // -rwxr-xr-x 2 kkocdko kkocdko 6973600 Feb  7 16:24 target/release/sqlite-bench

            // let (send, response) = tokio::sync::oneshot::channel();
            // self.tx
            //     .send(Box::new(move |s| {
            //         // f may be inlined, it's fine
            //         let _ = send.send(f(s));
            //     }))
            //     .await
            //     .unwrap();
            // response.await.unwrap()
            // -rwxr-xr-x 2 kkocdko kkocdko 6977832 Feb  7 16:14 target/release/sqlite-bench
        }
    }
}

fn bench_spawn_blocking_vs_mono() {
    async fn inner() {
        // use mono::Mono;
        use mono::MonoM as Mono;
        static DB: LazyLock<Mono<Connection>> = LazyLock::new(|| {
            let db = rusqlite::Connection::open_in_memory().unwrap();
            db.execute(
                "CREATE TABLE dav_users (uid BLOB PRIMARY KEY, auth BLOB UNIQUE)",
                (),
            )
            .unwrap();
            Mono::new(db)
        });
        let mut h = tokio::task::JoinSet::new();
        for i in 0..16 {
            h.spawn(async move {
                let sep = 8 * SCALE;
                for j in (i * sep)..((i + 1) * sep) {
                    DB.call(move |db| {
                        let uid = j.to_string();
                        let auth = uid.to_string() + "_auth";

                        // assert!(auth.starts_with(&uid));
                        // assert!(db.changes() == 0);

                        let mut stmd = db
                            .prepare_cached("REPLACE INTO dav_users VALUES (?, ?)")
                            .unwrap();
                        stmd.execute((&uid, &auth)).unwrap();
                        let mut stmd = db
                            .prepare_cached("SELECT uid FROM dav_users WHERE auth = ?")
                            .unwrap();
                        let mut rows = stmd.query((&auth,)).unwrap();
                        let row = rows.next().unwrap().unwrap();
                        let uid_queried: String = row.get(0).unwrap();
                        assert_eq!(uid, uid_queried);
                    })
                    .await;
                }
            });
        }
        while let Some(r) = h.join_next().await {
            r.unwrap();
        }
    }
    let begin = std::time::Instant::now();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(inner());
    println!(
        "> bench_spawn_blocking_vs_mono = {}",
        begin.elapsed().as_millis()
    );
}

fn bench_create_index_vs_unique() {
    fn inner(is_unique: bool) {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        if is_unique {
            db.execute(
                "CREATE TABLE IF NOT EXISTS dav_users (uid BLOB PRIMARY KEY, auth BLOB UNIQUE)",
                (),
            )
            .unwrap();
        } else {
            db.execute(
                "CREATE TABLE IF NOT EXISTS dav_users (uid BLOB PRIMARY KEY, auth BLOB)",
                (),
            )
            .unwrap();
            db.execute(
                "CREATE INDEX IF NOT EXISTS dav_users_index on dav_users (auth)",
                (),
            )
            .unwrap();
        }
        for i in 0..400000 {
            let uid = i.to_string();
            let auth = uid.to_string() + "_auth";

            let mut stmd = db
                .prepare_cached("REPLACE INTO dav_users VALUES (?, ?)")
                .unwrap();
            stmd.execute((&uid, &auth)).unwrap();

            let mut stmd = db
                .prepare_cached("SELECT uid FROM dav_users WHERE auth = ?")
                .unwrap();
            let mut rows = stmd.query((&auth,)).unwrap();
            let row = rows.next().unwrap().unwrap();
            let uid_queried: String = row.get(0).unwrap();
            assert_eq!(uid, uid_queried);
        }
    }

    let begin = std::time::Instant::now();
    inner(true);
    println!(
        "> bench_create_index_vs_unique:unique = {}",
        begin.elapsed().as_millis()
    );

    let begin = std::time::Instant::now();
    inner(false);
    println!(
        "> bench_create_index_vs_unique:create_index = {}",
        begin.elapsed().as_millis()
    );
}

fn main() {
    bench_spawn_blocking_vs_mono();
    // bench_create_index_vs_unique();
}

// cargo run --release
