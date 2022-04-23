mod services;
use axum::Router;
use std::env;
use std::io;
use std::net::SocketAddr;
use std::process;
use std::sync::Mutex;
use std::thread;

lazy_static::lazy_static! {
    pub static ref DB: Mutex<rusqlite::Connection> = {
        let path = env::current_exe().unwrap().with_extension("db");
        let db = rusqlite::Connection::open(path).unwrap();
        Mutex::new(db)
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    thread::spawn(|| loop {
        println!("type ':q' to quit");
        let input = &mut String::new();
        io::stdin().read_line(input).unwrap();
        if input.trim() == ":q" {
            process::exit(0);
        }
    });

    let app = Router::new()
        .route("/", services::home::main())
        .route("/paste", services::paste::main())
        .route("/qqbot", services::qqbot::main())
        .into_make_service();
    let addr = SocketAddr::from(([0, 0, 0, 0], 9304));
    println!("listening on {addr}");
    axum::Server::bind(&addr).serve(app).await?;
    Ok(())
}
