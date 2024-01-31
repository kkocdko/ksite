use crate::utils::{LazyLock, Mono};
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

fn file_path() -> PathBuf {
    std::env::current_exe().unwrap().with_extension("db")
}

pub static DB: LazyLock<Mono<Connection>> = LazyLock::new(|| {
    // "/home/kkocdko/misc/code/ksite/.vscode/bak/ksite.db".into()
    let db = Connection::open(file_path()).unwrap();
    // https://www.sqlite.org/speed.html
    // https://www.sqlite.org/optoverview.html
    // https://www.sqlite.org/pragma.html#pragma_journal_mode
    // https://www.sqlite.org/withoutrowid.html
    // https://www.sqlite.org/pragma.html#pragma_optimize
    // https://www.sqlite.org/mmap.html
    // https://github.com/rusqlite/rusqlite/issues/393#issuecomment-1313587354
    // https://www.powersync.co/blog/sqlite-optimizations-for-ultra-high-performance
    // https://crates.io/crates/r2d2
    // https://github.com/actix/examples/blob/0be798cdd23f2adb3ca9f1bf6708921ffb8e14d2/databases/sqlite/src/main.rs
    let sqls = [
        // improve writing by `WAL` mode, the `TRUNCATE` is alternative
        "PRAGMA journal_mode=WAL",
        // safe for app crashes, but might become corrupted if the os crashes
        "PRAGMA synchronous=OFF",
        // we don't need to touch db file during program execution
        "PRAGMA locking_mode=EXCLUSIVE",
    ];
    for sql in sqls {
        match db.execute(sql, ()) {
            Ok(_) | Err(rusqlite::Error::ExecuteReturnedResults) => 0,
            e => e.unwrap(),
        };
    }
    Mono::new(db)
});

pub async fn backup() {
    Mono::call(&DB, move |db| {
        // shrink
        db.execute("PRAGMA journal_mode=TRUNCATE", ()).unwrap();
        db.execute("VACUUM", ()).unwrap();
        db.execute("PRAGMA journal_mode=WAL", ()).unwrap();
    })
    .await;
    std::fs::copy(
        file_path(),
        file_path().with_extension(format!("{}.db", UNIX_EPOCH.elapsed().unwrap().as_secs())),
    )
    .unwrap();
}
