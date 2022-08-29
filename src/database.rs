use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::sync::Mutex;

/// # Use `db!()` macro instead of access directly!
pub static DB_: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let path = std::env::current_exe().unwrap().with_extension("db");
    let db = Connection::open(path).unwrap();

    // Optimize for Performance
    // https://www.sqlite.org/speed.html
    // https://www.sqlite.org/pragma.html

    // The `WAL` mode will improve writing but slow down reading a little.
    db.pragma_update(None, "journal_mode", "TRUNCATE").unwrap();
    // db.pragma_update(None, "journal_mode", "WAL").unwrap();

    // Sync less often than `FULL` and still safe enough.
    db.pragma_update(None, "synchronous", "NORMAL").unwrap();

    // We don't need to touch db file during program execution.
    db.pragma_update(None, "locking_mode", "EXCLUSIVE").unwrap();

    Mutex::new(db)
});

#[macro_export]
macro_rules! db {
    // simplest usage
    ( $sql:expr ) => {{ $crate::db!($sql, []) }};
    // execute a statement with params
    ( $sql:expr, [ $($param:tt)* ] ) => {{
        let db = $crate::database::DB_.lock().unwrap();
        let mut stmd = db.prepare_cached($sql).unwrap();
        stmd.execute(rusqlite::params![$($param)*])
    }};
    // execute a statement then returns `last_insert_rowid()`
    ( $sql:expr, [ $($param:tt)* ], & ) => {{
        let db = $crate::database::DB_.lock().unwrap();
        let mut stmd = db.prepare_cached($sql).unwrap();
        stmd.execute(rusqlite::params![$($param)*]).map(|_| db.last_insert_rowid())
    }};
    // query one row, the symbol '^' means "match first" in regexp
    ( $sql:expr, [ $($param:tt)* ], ^$f:expr ) => {{
        let db = $crate::database::DB_.lock().unwrap();
        let mut stmd = db.prepare_cached($sql).unwrap();
        stmd.query_row(rusqlite::params![$($param)*], $f)
    }};
    // query and save all the results into a `Vec<T>`
    ( $sql:expr, [ $($param:tt)* ], $f:expr ) => {(||{
        let mut ret = Vec::new();
        let db = $crate::database::DB_.lock().unwrap();
        let mut stmd = db.prepare_cached($sql).unwrap();
        for v in stmd.query_map(rusqlite::params![$($param)*], $f).unwrap() {
            ret.push(v?);
        }
        std::result::Result::<_, rusqlite::Error>::Ok(ret)
    })()};
}
