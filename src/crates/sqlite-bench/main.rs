//！ benchmarks

// SELECT * FROM sqlite_master WHERE type = 'index';

use std::sync::OnceLock;

/// While [`std::sync::LazyLock`](https://doc.rust-lang.org/stable/std/sync/struct.LazyLock.html) is still not in stable.
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

pub mod mono1 {
    use std::sync::Mutex;
    pub struct Mono<I> {
        inner: Mutex<I>,
    }
    impl<I: Send + 'static> Mono<I> {
        pub fn new(init: I) -> Self {
            Self {
                inner: Mutex::new(init),
            }
        }
        pub async fn call<T: Send + 'static>(
            &'static self,
            f: impl Fn(&mut I) -> T + Send + 'static,
        ) -> T {
            let inner = &self.inner;
            tokio::task::spawn_blocking(move || {
                let mut inner = inner.lock().unwrap();
                let inner = &mut *inner;
                f(inner)
            })
            .await
            .unwrap()
        }
    }
}

fn bench_spawn_blocking_vs_mono() {
    async fn inner() {
        use std::sync::Arc;
        use tokio::sync::Mutex;
        use tokio::sync::Semaphore;
        let db = rusqlite::Connection::open_in_memory().unwrap();
        db.execute(
            "CREATE TABLE IF NOT EXISTS dav_users (uid BLOB PRIMARY KEY, auth BLOB UNIQUE)",
            (),
        )
        .unwrap();
        let semaphore = Arc::new(Semaphore::new(0));
        let db = Arc::new(Mutex::new(db));
        const COUNT: u32 = 16;
        for _ in 0..COUNT {
            let semaphore_cur = Arc::clone(&semaphore);
            let db_cur = Arc::clone(&db);
            // tokio::spawn(async move {
            //     for i in 0..200000 {
            //         let db = db_cur.lock().await;
            //         tokio::task::spawn_blocking(move || {
            //             let uid = i.to_string();
            //             let auth = uid.to_string() + "_auth";

            //             let mut stmd = db
            //                 .prepare_cached("REPLACE INTO dav_users VALUES (?, ?)")
            //                 .unwrap();
            //             stmd.execute((&uid, &auth)).unwrap();

            //             let mut stmd = db
            //                 .prepare_cached("SELECT uid FROM dav_users WHERE auth = ?")
            //                 .unwrap();
            //             let mut rows = stmd.query((&auth,)).unwrap();
            //             let row = rows.next().unwrap().unwrap();
            //             let uid_queried: String = row.get(0).unwrap();
            //             assert_eq!(uid, uid_queried);
            //         })
            //         .await
            //         .unwrap();
            //     }
            //     semaphore_cur.add_permits(1);
            // });
        }
        semaphore.acquire_many(COUNT).await.unwrap().forget();
    }
    let begin = std::time::Instant::now();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(inner());
    println!(
        "> bench_spawn_blocking_vs_mono:unique = {}",
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
    {
        // use mono1::Mono;
        // static DB: LazyLock<Mono<String>> = LazyLock::new(|| Mono::new(String::new()));
        // Mono::call(&DB, |db| {
        //     //
        // }).await;
    }
    // bench_spawn_blocking_vs_mono();
}

// cargo run --release
