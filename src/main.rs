mod units;
use axum::Router;
use std::env;
use std::io;
use std::net::SocketAddr;
use std::process;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    thread::spawn(|| loop {
        let input = &mut String::new();
        io::stdin().read_line(input).unwrap();
        if input.trim() == ":q" {
            process::exit(0);
        }
    });

    let server = async {
        let addr = SocketAddr::from(([0, 0, 0, 0], 9304));
        println!("server addr = {addr}");

        let app = Router::new()
            .merge(units::health::service())
            .merge(units::paste::service())
            .merge(units::qqbot::service())
            .merge(units::welcome::service())
            .into_make_service();
        axum::Server::bind(&addr).serve(app).await
    };

    let oscillator = async {
        let interval = Duration::from_secs(60);
        println!("oscillator interval = {:?}", &interval);

        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            tokio::join!(units::health::tick(), units::qqbot::tick());
        }
    };

    let _ = tokio::join!(server, oscillator);
}

lazy_static::lazy_static! {
    /// Use `db!()` macro instead of access directly
    pub static ref DATABASE: Mutex<rusqlite::Connection> = {
        let path = env::current_exe().unwrap().with_extension("db");
        let db = rusqlite::Connection::open(path).unwrap();
        Mutex::new(db)
    };
}

#[macro_export]
macro_rules! db {
    ($sql:expr) => {{
        { crate::DATABASE.lock().unwrap() }
            .prepare_cached($sql)
            .and_then(|mut s| s.execute([]))
    }};
    ($sql:expr, [ $($param:tt)* ] ) => {{
        { crate::DATABASE.lock().unwrap() }
            .prepare_cached($sql)
            .and_then(|mut s| s.execute(rusqlite::params![$($param)*]))
    }};
    ($sql:expr, [ $($param:tt)* ], ( $( $idx:expr ),* ) ) => {(||{
        let database = crate::DATABASE.lock().unwrap();
        let mut stmd = database.prepare_cached($sql)?;
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
