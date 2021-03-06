mod auth;
mod database;
mod ticker;
mod tls;
// mod tls_next;
mod units;
mod utils;
use axum::{Router, Server};
use std::io;
use std::net::SocketAddr;
use std::process;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("[ksite v{}]", env!("CARGO_PKG_VERSION"));
    println!("enter :q to quit");
    println!("authorization token = {}", *auth::TOKEN);

    thread::spawn(|| loop {
        let buf = &mut String::new();
        if io::stdin().read_line(buf).is_ok() && buf.trim() == ":q" {
            println!("quit ksite");
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
            .merge(units::magazine::service())
            .merge(units::paste::service())
            .merge(units::qqbot::service())
            .merge(units::record::service())
            .merge(units::welcome::service())
            .into_make_service();
        // .into_make_service_with_connect_info::<SocketAddr>();

        // let server = Server::bind(&addr).serve(app);
        let server = Server::builder(tls::incoming(&addr)).serve(app);

        server.await.unwrap();
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

    tokio::join!(server, oscillator);
}
