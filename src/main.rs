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

// #[global_allocator]
// static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc; // or rpmalloc::RpMalloc

async fn aa() {
    use std::sync::Mutex;
    // static LIST: [(&str, Mutex<String>); -(0 - 1 - 1) as _] = [
    //     ("a", Mutex::new(String::new())),
    //     ("b", Mutex::new(String::new())),
    // ];
    macro_rules! foo {
        ( $($y:expr),+ ) => {
            const fn ret_one<T>(_:&T)->i32{1}
            const fn ret_mutex<T>(_:&T)->Mutex<String>{Mutex::new(String::new())}
            const LEN: usize = -(-$( ret_one(&$y) )-+) as _;
            static LAST: [Mutex<String>; LEN] = [$( ret_mutex(&$y) ),+];
            async fn what(i:usize){
                let last = LAST[i].lock().unwrap();
            }
            tokio::join!(
                $( what(ret_one(&$y) as _) ),+
            );
        };
    }
    foo!(1, 1, 4, 5, 1, 4);
    // https://github.com/rust-lang/rust/issues/83527
}

fn main() {
    launcher::launch(run);
}

async fn run() {
    // return ticker::test2();
    // return ticker::fuzzle_test().await;
    // return units::paste_next::dev().await;

    log!("crate::run");

    // db_upgrade(); // uncomment this if we need to upgrade database

    let server = async {
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
        log!("auth key = {}", auth::auth_key());
        let addr = SocketAddr::from(([0, 0, 0, 0], 9304)); // server address here
        log!("server address = {addr}");
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
        // tls::serve(&addr, app).await;
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
            units::qqbot::tick().await;
            interval.tick().await;
            care!(tokio::time::timeout(TIMEOUT, tasks()).await).ok();
            // let stamp = httpdate::fmt_http_date(std::time::SystemTime::now());
            // log!("oscillator loop bottom, at {stamp}");
        }
    };

    tokio::join!(server, oscillator);
}

/// Deal with database upgrade.
#[cfg(feature = "db-upgrade")]
fn db_upgrade() {
    const CURRENT_VER: &str = env!("CARGO_PKG_VERSION");
    fn db_set(k: &str, v: &[u8]) {
        db!("REPLACE INTO admin VALUES (?, ?)", [k, v]).unwrap();
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
