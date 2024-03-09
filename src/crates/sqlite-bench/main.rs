//！ benchmarks

// https://github.com/the-lean-crate/criner/issues/1
// https://github.com/rusqlite/rusqlite/issues/393#issuecomment-982806506

use rusqlite::Connection;
use std::mem::MaybeUninit;
use std::sync::OnceLock;

const SCALE: u64 = 1000;

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

/// Use in async environment, determine the fastest way.
fn bench_parallel_async() {
    fn init_db(db: &mut Connection) {
        db.execute(
            "CREATE TABLE dav_users (uid BLOB PRIMARY KEY, auth BLOB UNIQUE)",
            (),
        )
        .unwrap();
    }
    fn rw_once(db: &mut Connection, j: u64) {
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
    }

    pub mod mono {
        use std::sync::Mutex;

        pub struct MonoSpawnBlocking<I>(Mutex<I>);
        impl<I: Send + 'static> MonoSpawnBlocking<I> {
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

        pub struct Mono<T> {
            #[allow(clippy::type_complexity)]
            tx: tokio::sync::mpsc::Sender<Box<dyn FnOnce(&mut T) + Send>>,
        }

        impl<T: Send + 'static> Mono<T> {
            pub fn new(mut v: T) -> Self {
                let (tx, mut rx) = tokio::sync::mpsc::channel::<Box<dyn FnOnce(&mut T) + Send>>(1);
                std::thread::spawn(move || {
                    // after self.tx drop, the recv() here will cause thread exit, without memory leaking
                    while let Some(f) = rx.blocking_recv() {
                        f(&mut v);
                    }
                });
                Self { tx }
            }

            pub async fn call<'env, R: Send + 'env>(
                &self,
                f: impl FnOnce(&mut T) -> R + Send + 'env,
            ) -> R {
                let mutex = std::sync::Arc::new(tokio::sync::Mutex::const_new(None));
                let mut guard = mutex.clone().lock_owned().await;
                // SAFETY: Absolute crazy, but we ensure that `f` has ended by `mutex.lock().await`, so `f` won't be used after dropped at the end of current function
                let cb: Box<dyn FnOnce(&mut T) + Send> =
                    Box::new(move |s: &mut _| *guard = Some(f(s))); // f may be inlined, it's fine
                let cb: Box<dyn FnOnce(&mut T) + Send + 'static> = unsafe {
                    let raw_ptr = Box::into_raw(cb);
                    Box::from_raw(std::mem::transmute(raw_ptr)) //  what the fuck
                };
                self.tx.send(cb).await.unwrap();
                let mut guard = mutex.lock().await;
                guard.take().unwrap()
            }
        }
    }

    let begin = std::time::Instant::now();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            use mono::Mono;
            static DB: LazyLock<Mono<Connection>> = LazyLock::new(|| {
                let mut db = Connection::open_in_memory().unwrap();
                init_db(&mut db);
                Mono::new(db)
            });
            let mut h = tokio::task::JoinSet::new();
            for i in 0..16 {
                h.spawn(async move {
                    let sep = 8 * SCALE;
                    for j in (i * sep)..((i + 1) * sep) {
                        DB.call(|db| rw_once(db, j)).await;
                    }
                });
            }
            while let Some(r) = h.join_next().await {
                r.unwrap();
            }
            DB.call(|db| {
                #[allow(invalid_value)]
                let mut blank = unsafe { MaybeUninit::zeroed().assume_init() }; // safety: it's static, we never drop it
                std::mem::swap(db, &mut blank);
            })
            .await;
        });
    println!(
        "> bench_parallel_async:mono = {}",
        begin.elapsed().as_millis()
    );

    let begin = std::time::Instant::now();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            use mono::MonoSpawnBlocking as Mono;
            static DB: LazyLock<Mono<Connection>> = LazyLock::new(|| {
                let mut db = Connection::open_in_memory().unwrap();
                init_db(&mut db);
                Mono::new(db)
            });
            let mut h = tokio::task::JoinSet::new();
            for i in 0..16 {
                h.spawn(async move {
                    let sep = 8 * SCALE;
                    for j in (i * sep)..((i + 1) * sep) {
                        DB.call(move |db| rw_once(db, j)).await;
                    }
                });
            }
            while let Some(r) = h.join_next().await {
                r.unwrap();
            }
            DB.call(|db| {
                #[allow(invalid_value)]
                let mut blank = unsafe { MaybeUninit::zeroed().assume_init() };
                std::mem::swap(db, &mut blank);
            })
            .await;
        });
    println!(
        "> bench_parallel_async:spawn_blocking = {}",
        begin.elapsed().as_millis()
    );

    #[allow(unused)]
    let begin = std::time::Instant::now();
    #[allow(unreachable_code)]
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            return; // the rdr2-sqlite with multi thread will cause sqlite_busy error
            let pool = r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::memory()).unwrap();
            init_db(&mut pool.get().unwrap());
            let mut h = tokio::task::JoinSet::new();
            for i in 0..16 {
                let pool = pool.clone();
                h.spawn(async move {
                    let sep = 8 * SCALE;
                    for j in (i * sep)..((i + 1) * sep) {
                        rw_once(&mut pool.get().unwrap(), j);
                    }
                });
            }
            while let Some(r) = h.join_next().await {
                r.unwrap();
            }
        });
    // println!(
    //     "> bench_parallel_async:rdr2 = {}",
    //     begin.elapsed().as_millis()
    // );

    #[allow(unused)]
    async fn bench_str_async() {
        use mono::Mono;
        let data = Mono::new(String::from("01234567"));
        let append = String::from("abcdefgh");
        // the goal: borrow vars, send closure to worker thread, receive returns by await
        tokio::spawn(async move {
            let out_fut = data.call(|d| {
                let c = d.pop();
                if d.is_empty() {
                    *d += &append;
                }
                c
            });
            let out = out_fut.await;
            dbg!(out);
        });
        // dbg!(&append); // append is moved, ensured
    }
    // tokio::spawn(bench_str_async());
}

/// The `UNIQUE` constraint will create index automatically, compare it's performance with `CREATE INDEX`.
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
    bench_parallel_async();
    bench_create_index_vs_unique();
}

// SELECT * FROM sqlite_master WHERE type = 'index';

// cargo run --release
