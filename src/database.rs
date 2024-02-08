use crate::utils::{LazyLock, Mono};
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

fn file_path() -> PathBuf {
    std::env::current_exe().unwrap().with_extension("db")
}

fn db_exec(db: &Connection, sql: &str) {
    match db.execute(sql, ()) {
        Ok(_) | Err(rusqlite::Error::ExecuteReturnedResults) => 0,
        e => e.unwrap(),
    };
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
    db_exec(&db, "PRAGMA journal_mode=WAL"); // improve writing by `WAL` mode, the `TRUNCATE` is alternative
    db_exec(&db, "PRAGMA synchronous=OFF"); // safe for app crashes, but might become corrupted if the os crashes
    db_exec(&db, "PRAGMA locking_mode=EXCLUSIVE"); // we don't need to touch db file during program execution
    Mono::new(db)
});

pub async fn backup() {
    DB.call(move |db| {
        // shrink
        db_exec(&db, "PRAGMA journal_mode=TRUNCATE");
        db_exec(&db, "VACUUM");
        db_exec(&db, "PRAGMA journal_mode=WAL");
    })
    .await;
    std::fs::copy(
        file_path(),
        file_path().with_extension(format!("{}.db", UNIX_EPOCH.elapsed().unwrap().as_secs())),
    )
    .unwrap();
}
