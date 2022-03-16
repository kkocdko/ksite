pub mod db;
mod units;
use axum::Router;
use std::io;
use std::net::SocketAddr;
use std::process;
use std::thread;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", units::home::service())
        .route("/paste", units::paste::service())
        .route("/qqbot", units::qqbot::service())
        .into_make_service();
    let addr: SocketAddr = "127.0.0.1:9304".parse().unwrap();
    println!("listening on {}", addr);
    thread::spawn(|| loop {
        println!("type \":q\" to quit");
        let input = &mut String::new();
        io::stdin().read_line(input).unwrap();
        if input.trim() == ":q" {
            process::exit(0);
        }
    });
    axum::Server::bind(&addr).serve(app).await.unwrap();
}
