use once_cell::sync::Lazy;
use std::env;
use std::sync::Mutex;

/// ## Use `db!()` macro instead of access directly!
pub static _DB: Lazy<Mutex<rusqlite::Connection>> = Lazy::new(|| {
    let path = env::current_exe().unwrap().with_extension("db");
    let db = rusqlite::Connection::open(path).unwrap();
    db.pragma_update(None, "synchronous", "OFF").unwrap();
    db.pragma_update(None, "locking_mode", "EXCLUSIVE").unwrap();
    Mutex::new(db)
});

#[macro_export]
macro_rules! db {
    ($sql:expr) => {{
        { crate::database::_DB.lock().unwrap() }
            .prepare_cached($sql)
            .and_then(|mut s| s.execute([]))
    }};
    ($sql:expr, [ $($param:tt)* ] ) => {{
        { crate::database::_DB.lock().unwrap() }
            .prepare_cached($sql)
            .and_then(|mut s| s.execute(rusqlite::params![$($param)*]))
    }};
    ($sql:expr, [ $($param:tt)* ], ( $( $idx:expr ),* ) ) => {(||{
        let db = crate::database::_DB.lock().unwrap();
        let mut stmd = db.prepare_cached($sql)?;
        let mut ret = Vec::new();
        for entry in stmd.query_map(
            rusqlite::params![$($param)*],
            |r| Ok(( $( r.get($idx)?, )* ))
        )? {
            ret.push(entry?);
        }
        Result::<_, rusqlite::Error>::Ok(ret)
    })()};
}
