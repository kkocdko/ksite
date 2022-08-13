use once_cell::sync::Lazy;
use std::sync::Mutex;

/// # Use `db!()` macro instead of access directly!
pub static DB_: Lazy<Mutex<rusqlite::Connection>> = Lazy::new(|| {
    let path = std::env::current_exe().unwrap().with_extension("db");
    let db = rusqlite::Connection::open(path).unwrap();

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
    ( $sql:expr ) => {{ $crate::db!($sql, []) }};
    ( $sql:expr, [ $($param:tt)* ] ) => {{
        { $crate::database::DB_.lock().unwrap() }
            .prepare_cached($sql)
            .and_then(|mut s| s.execute(rusqlite::params![$($param)*]))
    }};
    ( $sql:expr, [ $($param:tt)* ], $f:expr ) => {(||{
        let mut ret = Vec::new();
        let db = $crate::database::DB_.lock().unwrap();
        let mut stmd = db.prepare_cached($sql)?;
        for v in stmd.query_map(rusqlite::params![$($param)*], $f)? {
            ret.push(v?);
        }
        std::result::Result::<_, rusqlite::Error>::Ok(ret)
    })()};
}
