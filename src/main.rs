mod units;
use axum::Router;
use lazy_static::lazy_static;
use std::env;
use std::net::SocketAddr;
use std::process;
use tokio::sync::Mutex;
use std::thread;
use std::io;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", units::home::service())
        .route("/paste", units::paste::service().await)
        .into_make_service();
    let addr: SocketAddr = "127.0.0.1:9304".parse().unwrap();
    println!("listening on {}", addr);
    thread::spawn(||{
        println!("press <enter> key to exit");
        io::stdin().read_line(&mut String::new()).unwrap();
        process::exit(0);
    });
    axum::Server::bind(&addr).serve(app).await.unwrap();
}

// async fn create_user(
//     // this argument tells axum to parse the request body
//     // as JSON into a `CreateUser` type
//     Json(payload): Json<CreateUser>,
// ) -> impl IntoResponse {
//     // insert your application logic here
//     let user = User {
//         id: 1337,
//         username: payload.username,
//     };
//     // this will be converted into a JSON response
//     // with a status code of `201 Created`
//     (StatusCode::CREATED, Json(user))
// }
