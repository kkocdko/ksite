//! TLS & HTTPS support for the server.
use crate::db;
use axum::routing::IntoMakeService;
use axum::Router;
use futures_util::future::poll_fn;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, Http};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tower::MakeService;

/// Serve the services.
///
/// # Example
///
/// ```
/// use axum::{routing::get, Router};
/// let addr = SocketAddr::from(([0, 0, 0, 0], 9304));
/// let app = Router::new()
///     .route("/", get(|| async { "hi" }))
///     .into_make_service();
/// tls::serve(&addr, app).await;
/// ```
///
/// # About the performance and hyper's `Accept` trait:
///
/// ## 1. Why a slow TLS handshake will block other connections?
///
/// It's certainly that accepting new connection on one port should always in the main thread.
/// If you create a struct with `Accept` trait and use it in `Server::builder(your_struct)`,
/// you will realize that the code about TLS handshake in `fn poll_accept()` will running in the
/// main thread, so it blocks other connections and caused a bad performance.
///
/// ## 2. How to solve this problem?
///
/// Don't use `Accept` trait. Just write a loop and process the accepts manually. Move the code
/// about TLS handshake into `tokio::spawn(async { })`. Hyper's team was realized and wrote this in
/// their [1.0 roadmap](https://github.com/hyperium/hyper/blob/v0.14.20/docs/ROADMAP.md#higher-level-client-and-server-problems).
///
/// # Thanks to:
///
/// https://github.com/hyperium/hyper/blob/v0.14.20/src/server/server.rs#L176
/// https://github.com/tokio-rs/axum/tree/axum-v0.5.15/examples/low-level-rustls
/// https://github.com/programatik29/axum-server
pub async fn serve(addr: &SocketAddr, mut app: IntoMakeService<Router>) {
    fn db_get(k: &str) -> Vec<u8> {
        let r = db!("SELECT v FROM admin WHERE k = ?", [k], |r| r.get(0));
        r.unwrap().pop().unwrap()
    }

    // load cert and key from db
    let certs = vec![Certificate(db_get("ssl_cert"))];
    let key = PrivateKey(db_get("ssl_key"));
    let mut rustls_cfg = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth() // TODO: vertify? see warp's source code?
        .with_single_cert(certs, key)
        .unwrap();
    // enable http2, needs hyper / axum feature "http2"
    rustls_cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let acceptor = TlsAcceptor::from(Arc::new(rustls_cfg));
    let mut listener = AddrIncoming::bind(addr).unwrap();

    loop {
        let acceptor = acceptor.clone(); // cheap, Arc inner
        let mut stream = match poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx)).await {
            Some(Ok(v)) => v,
            _ => continue, // ignore error here
        };

        // HTTP to HTTPS
        let mut flag = [0]; // expect 0x16, the tls handshake mark
        let mut buf = tokio::io::ReadBuf::new(&mut flag);
        poll_fn(|cx| stream.poll_peek(cx, &mut buf)).await.ok();
        if flag[0] != 0x16 {
            const PAGE: &[u8] = b"HTTP/1.1 200 OK\r\ncontent-type:text/html\r\n\r\n<script>location=location.href.replace(':','s:')</script>\r\n\r\n\0";
            stream.write_all(PAGE).await.ok();
            stream.shutdown().await.ok(); // remember to close stream
            continue;
        }

        let svc = app.make_service(&stream);
        tokio::spawn(async move {
            let timeout = Duration::from_millis(2000);
            let accept = acceptor.accept(stream);
            if let Ok(Ok(stream)) = tokio::time::timeout(timeout, accept).await {
                let _ = Http::new()
                    .serve_connection(stream, svc.await.unwrap())
                    .await;
            }
        });
    }
}
