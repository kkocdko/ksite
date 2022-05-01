mod auth;
mod database;
mod ticker;
mod units;
use axum::Router;
use std::io;
use std::net::SocketAddr;
use std::process;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("[ksite]");
    println!("enter `:q` to shutdown");
    println!("auth token = {}", *auth::TOKEN);

    thread::spawn(|| loop {
        let input = &mut String::new();
        if io::stdin().read_line(input).is_ok() && input.trim() == ":q" {
            process::exit(0);
        }
    });

    let server = async {
        let addr = SocketAddr::from(([0, 0, 0, 0], 9304));
        println!("server addr = {addr}");

        let app = Router::new()
            .merge(units::chat::service())
            .merge(units::health::service())
            .merge(units::paste::service())
            .merge(units::qqbot::service())
            .merge(units::welcome::service())
            .into_make_service();
        axum::Server::bind(&addr).serve(app).await.unwrap();
    };

    let oscillator = async {
        let interval = Duration::from_secs(60);
        println!("oscillator interval = {:?}", &interval);

        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            let _ = tokio::join!(
                tokio::spawn(units::health::tick()),
                tokio::spawn(units::qqbot::tick()),
            );
        }
    };

    tokio::join!(server, oscillator);
}
