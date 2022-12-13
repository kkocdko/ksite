use once_cell::sync::Lazy;
use rusqlite::{Connection, Row, ToSql};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

fn file_path() -> PathBuf {
    std::env::current_exe().unwrap().with_extension("db")
}

fn load() -> Connection {
    let db = Connection::open(file_path()).unwrap();

    // Optimize for Performance
    // https://www.sqlite.org/speed.html
    // https://www.sqlite.org/pragma.html#pragma_journal_mode
    // https://www.sqlite.org/withoutrowid.html
    // https://www.sqlite.org/pragma.html#pragma_optimize

    // The `WAL` mode will improve writing but slow down reading a little.
    db.pragma_update(None, "journal_mode", "WAL").unwrap();

    // Sync less often than `FULL` and still safe enough.
    db.pragma_update(None, "synchronous", "NORMAL").unwrap();

    // We don't need to touch db file during program execution.
    db.pragma_update(None, "locking_mode", "EXCLUSIVE").unwrap();

    db
}

pub fn backup() {
    let mut db = DB.lock().unwrap();

    // merge wal file
    db.pragma_update(None, "journal_mode", "TRUNCATE").unwrap();

    // db.pragma_update(None, "optimize", "").unwrap();

    // shrink size
    db.execute_batch("VACUUM").unwrap();

    unsafe {
        // safety: we held the mutex in the whole period
        let db_ptr = std::ptr::addr_of_mut!(*db);
        db_ptr.drop_in_place(); // run Drop::drop but don't free memory

        std::fs::copy(
            file_path(),
            file_path().with_extension(format!("{}.db", UNIX_EPOCH.elapsed().unwrap().as_secs())),
        )
        .unwrap();

        db_ptr.write(load());
    };
}

/// # Use `db!()` macro instead of access directly!
pub mod inner_ {
    use super::*;

    pub static DB: Lazy<Mutex<Connection>> = Lazy::new(|| Mutex::new(load()));

    pub fn exec_batch(sqls: &str) -> rusqlite::Result<()> {
        let db = DB.lock().unwrap();
        db.execute_batch(sqls)
    }

    pub fn exec_param(sql: &str, params: &[&dyn ToSql]) -> rusqlite::Result<()> {
        let db = DB.lock().unwrap();
        let mut stmd = db.prepare_cached(sql)?;
        stmd.execute(params)?;
        Ok(())
    }

    // pub fn exec_param_lastid(sql: &str, params: &[&dyn ToSql]) -> rusqlite::Result<i64> {
    //     let db = DB.lock().unwrap();
    //     let mut stmd = db.prepare_cached(sql)?;
    //     stmd.execute(params)?;
    //     Ok(db.last_insert_rowid())
    // }

    pub fn query_row<T, F>(sql: &str, params: &[&dyn ToSql], f: F) -> rusqlite::Result<T>
    where
        F: FnOnce(&Row) -> rusqlite::Result<T>,
    {
        let db = DB.lock().unwrap();
        let mut stmd = db.prepare_cached(sql)?;
        stmd.query_row(params, f)
    }

    pub fn query_rows<T, F>(sql: &str, params: &[&dyn ToSql], mut f: F) -> rusqlite::Result<Vec<T>>
    where
        F: FnMut(&Row) -> rusqlite::Result<T>,
    {
        let db = DB.lock().unwrap();
        let mut stmd = db.prepare_cached(sql)?;
        let mut rows = stmd.query(params)?;
        let mut ret = Vec::new();
        while let Ok(Some(r)) = rows.next() {
            ret.push(f(r)?);
        }
        Ok(ret)
    }
}
use inner_::*; // not `pub use`

#[macro_export]
macro_rules! db {
    // execute without params and cache, supports multi statement
    ( $sqls:literal ) => {{
        $crate::database::inner_::exec_batch(
            $crate::strip_str!($sqls)
        )
    }};
    // execute with params
    ( $sql:literal, [ $($param:expr),* ] ) => {{
        $crate::database::inner_::exec_param(
            $crate::strip_str!($sql),
            rusqlite::params![$($param),*]
        )
    }};
    // execute and returns `last_insert_rowid()`
    // ( $sql:literal, [ $($param:expr),* ], & ) => {{
    //     $crate::database::inner_::exec_param_lastid(
    //         $crate::strip_str!($sql),
    //         rusqlite::params![$($param),*]
    //     )
    // }};
    // query and return the first row
    ( $sql:literal, [ $($param:expr),* ], &$f:expr ) => {{
        $crate::database::inner_::query_row(
            $crate::strip_str!($sql),
            rusqlite::params![$($param),*],
            $f
        )
    }};
    // query and return all rows as `Vec<T>`
    ( $sql:literal, [ $($param:expr),* ], $f:expr ) => {{
        $crate::database::inner_::query_rows(
            $crate::strip_str!($sql),
            rusqlite::params![$($param),*],
            $f
        )
    }};
}
