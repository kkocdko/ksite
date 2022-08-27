#![feature(const_slice_from_raw_parts)] // stabilized in 1.64 (#97522)
#![feature(future_poll_fn)] // stabilized in 1.64 (#99306)
mod auth;
mod database;
mod ticker;
mod tls;
mod units;
mod utils;
use axum::Router;
use std::io;
use std::net::SocketAddr;
use std::process;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("enter :q to quit");
    println!("authorization token = {}", *auth::TOKEN);

    // db_upgrade(); // uncomment this if we need to upgrade database

    thread::spawn(|| loop {
        let buf = &mut String::new();
        if io::stdin().read_line(buf).is_ok() && buf.trim() == ":q" {
            println!("quit");
            process::exit(0);
        }
        thread::sleep(Duration::from_secs(1));
    });

    let server = async {
        let addr = SocketAddr::from(([0, 0, 0, 0], 9304));
        println!("server address = {addr}");

        let app = Router::new()
            .merge(units::admin::service())
            .merge(units::chat::service())
            .merge(units::health::service())
            .merge(units::info::service())
            .merge(units::magazine::service())
            .merge(units::paste::service())
            // .merge(units::paste_next::service())
            .merge(units::qqbot::service())
            .into_make_service();
        // .into_make_service_with_connect_info::<SocketAddr>();

        // axum::Server::bind(&addr).serve(app).await.unwrap();
        tls::serve(&addr, app).await;
    };

    let oscillator = async {
        let interval = Duration::from_secs(60);
        println!("oscillator interval = {:?}", &interval);

        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            let _ = tokio::join!(
                units::health::tick(),
                units::magazine::tick(),
                units::qqbot::tick(),
            );
        }
    };

    // TODO: benchmark with tls enabled always failed on linux (but it's normal on windows)
    // seems a problem of rustls. any idea to fix this?
    // tokio::spawn(async {
    //     tokio::time::sleep(Duration::from_millis(1000)).await;
    //     loop {
    //         tokio::time::sleep(Duration::from_millis(500)).await;
    //         let a = utils::fetch_text("https://127.0.0.1:9304/info").await;
    //         dbg!(a).ok();
    //     }
    // });

    tokio::join!(server, oscillator);
}

/// Deal with database upgrade.
#[cfg(feature = "db_upgrade")]
fn db_upgrade() {
    let a = db!("SELECT * FROM admin WHERE k = ?", ["version"], |_| Ok(()));
    if matches!(a, Ok(v) if v.is_empty()) {
        println!(">>> upgrade database");

        db!(
            "REPLACE INTO admin VALUES (?1, ?2)",
            ["version", env!("CARGO_PKG_VERSION").as_bytes()]
        )
        .unwrap();

        // admin
        {
            db!("ALTER TABLE admin RENAME TO old_admin").unwrap();
            let rows: Vec<(String, Vec<u8>)> = db!("SELECT * FROM old_admin", [], |r| Ok((
                r.get(0)?,
                r.get(1)?,
            )))
            .unwrap();
            db!("CREATE TABLE admin (k TEXT PRIMARY KEY, v BLOB)").unwrap();
            for (k, v) in rows {
                fn db_set(k: &str, v: Vec<u8>) {
                    db!("REPLACE INTO admin VALUES (?1, ?2)", [k, v]).unwrap();
                }
                db_set(&k, v);
            }
            db!("DROP TABLE old_admin").unwrap();
        }

        // health
        {
            db!("ALTER TABLE health_list RENAME TO old_health_list").unwrap();
            let rows: Vec<(u64, String, String)> = db!(
                "SELECT * FROM old_health_list",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?,))
            )
            .unwrap();
            db!("CREATE TABLE health_list (id INTEGER PRIMARY KEY, token TEXT, body TEXT)")
                .unwrap();
            for (id, token, body) in rows {
                let sql = "REPLACE INTO health_list VALUES (?1, ?2, ?3)";
                db!(sql, [id, token, body]).unwrap();
            }
            db!("DROP TABLE old_health_list").unwrap();
        }

        // qqbot
        {
            {
                db!("ALTER TABLE qqbot_cfg RENAME TO old_qqbot_cfg").unwrap();
                let rows: Vec<(String, Vec<u8>)> = db!("SELECT * FROM old_qqbot_cfg", [], |r| Ok(
                    (r.get(0)?, r.get(1)?,)
                ))
                .unwrap();
                db!("CREATE TABLE qqbot_cfg (k TEXT PRIMARY KEY, v BLOB)").unwrap();
                for (k, v) in rows {
                    fn db_cfg_set(k: &str, v: Vec<u8>) {
                        db!("REPLACE INTO qqbot_cfg VALUES (?1, ?2)", [k, v]).unwrap();
                    }
                    db_cfg_set(&k, v);
                }
                db!("DROP TABLE old_qqbot_cfg").unwrap();
            }
            {
                db!("ALTER TABLE qqbot_groups RENAME TO old_qqbot_groups").unwrap();
                let rows: Vec<(i64)> =
                    db!("SELECT * FROM old_qqbot_groups", [], |r| Ok((r.get(0)?))).unwrap();
                db!("CREATE TABLE qqbot_groups (group_id INTEGER PRIMARY KEY)").unwrap();
                for (v) in rows {
                    pub fn db_groups_insert(group_id: i64) {
                        db!("REPLACE INTO qqbot_groups VALUES (?)", [group_id]).unwrap();
                    }
                    db_groups_insert(v);
                }
                db!("DROP TABLE old_qqbot_groups").unwrap();
            }
        }
    }
}
