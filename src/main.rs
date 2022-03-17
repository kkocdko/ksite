pub mod db;
mod units;
use axum::Router;
use std::env;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::process;
use std::thread;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", units::home::service())
        .route("/paste", units::paste::service())
        .route("/qqbot", units::qqbot::service())
        .into_make_service();
    let addr = match env::args().nth(1) {
        Some(v) => v.parse().unwrap(),
        None => SocketAddr::from((Ipv4Addr::LOCALHOST, 9304)),
    };
    println!("listening on {addr}");
    thread::spawn(|| loop {
        println!("type ':q' to quit");
        let input = &mut String::new();
        io::stdin().read_line(input).unwrap();
        if input.trim() == ":q" {
            process::exit(0);
        }
    });
    axum::Server::bind(&addr).serve(app).await?;
    Ok(())
}
