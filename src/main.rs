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
    // return units::paste_next::dev().await;
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("enter :q to quit");
    println!("auth key = {}", *auth::AUTH_KEY);

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
                // units::paste_next::tick(),
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
    const CURRENT_VER: &str = env!("CARGO_PKG_VERSION");
    fn db_set(k: &str, v: &[u8]) {
        db!("REPLACE INTO admin VALUES (?1, ?2)", [k, v]).unwrap();
    }
    fn db_get(k: &str) -> Option<(Vec<u8>,)> {
        db!("SELECT v FROM admin WHERE k = ?", [k], ^(0)).ok()
    }
    if !matches!(
        db_get("version"),
        Some((v,)) if v == CURRENT_VER.as_bytes()
    ) {
        println!("upgrade database structure to v{CURRENT_VER}");
        db_set("version", CURRENT_VER.as_bytes());
        db!("DROP TABLE health_list").unwrap();
    }
}
