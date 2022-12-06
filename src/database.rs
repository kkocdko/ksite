use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

fn get_db_file_path() -> PathBuf {
    std::env::current_exe().unwrap().with_extension("db")
}

fn load_db() -> Connection {
    let db = Connection::open(get_db_file_path()).unwrap();

    // Optimize for Performance
    // https://www.sqlite.org/speed.html
    // https://www.sqlite.org/pragma.html

    // The `WAL` mode will improve writing but slow down reading a little.
    db.pragma_update(None, "journal_mode", "TRUNCATE").unwrap();
    // db.pragma_update(None, "journal_mode", "WAL").unwrap();
    // TODO: use WAL mode, switch to TRUNCATE before backup

    // Sync less often than `FULL` and still safe enough.
    db.pragma_update(None, "synchronous", "NORMAL").unwrap();

    // We don't need to touch db file during program execution.
    db.pragma_update(None, "locking_mode", "EXCLUSIVE").unwrap();

    db
}

/// # Use `db!()` macro instead of access directly!
pub static DB_: Lazy<Mutex<Connection>> = Lazy::new(|| Mutex::new(load_db()));

pub fn backup() {
    let mut db = DB_.lock().unwrap();
    db.execute("VACUUM", []).unwrap();
    db.pragma_update(None, "journal_mode", "TRUNCATE").unwrap();
    unsafe {
        // safety: we held the mutex in the whole period
        let db_ptr = std::ptr::addr_of_mut!(*db);
        db_ptr.drop_in_place();

        fs::copy(
            get_db_file_path(),
            get_db_file_path()
                .with_extension(format!("{}.db", UNIX_EPOCH.elapsed().unwrap().as_secs())),
        )
        .unwrap();

        db_ptr.write(load_db());
    };
}

#[macro_export]
macro_rules! db {
    // simplest usage
    ( $sql:literal ) => {{ $crate::db!($sql, []) }};
    // execute a statement with params
    ( $sql:literal, [ $($param:expr),* ] ) => {{
        let params = rusqlite::params![$($param),*];
        let db = $crate::database::DB_.lock().unwrap();
        db.prepare_cached($crate::strip_str!($sql))
            .and_then(|mut s| s.execute(params))
    }};
    // execute a statement then returns `last_insert_rowid()`
    ( $sql:literal, [ $($param:expr),* ], & ) => {{
        let params = rusqlite::params![$($param),*];
        let db = $crate::database::DB_.lock().unwrap();
        db.prepare_cached($crate::strip_str!($sql))
            .and_then(|mut s| s.execute(params))
            .map(|_| db.last_insert_rowid())
    }};
    // query and return the first matched row, the symbol '^' means "first" in regexp
    ( $sql:literal, [ $($param:expr),* ], ^( $($idx:expr),* ) ) => {{
        let params = rusqlite::params![$($param),*];
        let db = $crate::database::DB_.lock().unwrap();
        db.prepare_cached($crate::strip_str!($sql))
            .and_then(|mut s| s.query_row(params, |r| Ok(( $( r.get($idx)?, )* ))))
    }};
    // query and return all rows as `Vec<T>`
    ( $sql:literal, [ $($param:expr),* ], ( $($idx:expr),* ) ) => {(||{
        let params = rusqlite::params![$($param),*];
        let mut ret = Vec::new();
        let db = $crate::database::DB_.lock().unwrap();
        let mut stmd = db.prepare_cached($crate::strip_str!($sql))?;
        let mut rows = stmd.query(params)?;
        while let Ok(Some(r)) = rows.next() {
            ret.push(( $( r.get($idx)?, )* ));
        }
        std::result::Result::<_, rusqlite::Error>::Ok(ret)
    })()};
}
