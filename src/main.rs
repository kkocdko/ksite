mod auth;
mod database;
// mod governor;
mod launcher;
mod ticker;
mod tls;
mod units;
mod utils;
use std::net::SocketAddr;
use std::time::Duration;
// use axum::middleware;

// #[global_allocator]
// static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc; // or rpmalloc::RpMalloc

fn main() {
    launcher::launch(run);
}

async fn run() {
    // return ticker::test2();
    // return ticker::fuzzle_test().await;
    // return units::paste_next::dev().await;
    log!("crate::run");
    log!("auth key = {}", auth::auth_key());

    // db_upgrade(); // uncomment this if we need to upgrade database

    let server = async {
        let addr = SocketAddr::from(([0, 0, 0, 0], 9304)); // server address here
        log!("server address = {addr}");

        let app = axum::Router::new()
            .merge(units::admin::service())
            .merge(units::chat::service())
            .merge(units::info::service())
            .merge(units::magazine::service())
            .merge(units::meet::service())
            // .merge(units::mirror::service())
            .merge(units::paste::service())
            // .merge(units::paste_next::service())
            // .merge(units::proxy::service())
            .merge(units::qqbot::service());
        // .layer(middleware::from_fn(governor::governor_layer))
        // .into_make_service_with_connect_info::<SocketAddr>();
        // axum::Server::bind(&addr).serve(app).await.unwrap();
        tls::serve(&addr, app).await;
    };

    let oscillator = async {
        const INTERVAL: Duration = Duration::from_secs(60);
        const TIMEOUT: Duration = Duration::from_secs(45);
        log!("oscillator interval = {INTERVAL:?}, timeout = {TIMEOUT:?}");
        async fn tasks() {
            tokio::join!(
                units::magazine::tick(),
                // units::paste_next::tick(),
                units::qqbot::tick()
            );
        }
        let mut interval = tokio::time::interval(INTERVAL);
        loop {
            interval.tick().await;
            care!(tokio::time::timeout(TIMEOUT, tasks()).await).ok();
            let stamp = httpdate::fmt_http_date(std::time::SystemTime::now());
            log!("oscillator loop bottom, at {stamp}");
        }
    };

    // let tasks = tokio::task::LocalSet::new();
    // tasks.spawn_local(units::magazine::tick());
    // tasks.spawn_local(units::qqbot::tick());
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
        log!("upgrade database structure to v{CURRENT_VER}");
        db_set("version", CURRENT_VER.as_bytes());
        db!("DROP TABLE health_list").unwrap();
    }
}
