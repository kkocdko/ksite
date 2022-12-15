//! TLS & HTTPS support for the server.

use crate::units::admin::db_get;
use axum::routing::Router;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, Http};
use std::future::poll_fn;
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;

/// Serve the services over TLS.
///
/// # Example
///
/// ```
/// use axum::{routing::get, Router};
/// let addr = SocketAddr::from(([0, 0, 0, 0], 9304));
/// let app = Router::new()
///     .route("/", get(|| async { "hi" }));
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
/// their [1.0 roadmap](
/// https://github.com/hyperium/hyper/blob/v0.14.20/docs/ROADMAP.md#higher-level-client-and-server-problems).
///
/// # Thanks to:
///
/// * https://github.com/hyperium/hyper/blob/v0.14.20/src/server/server.rs#L176
/// * https://github.com/tokio-rs/axum/tree/axum-v0.5.15/examples/low-level-rustls
/// * https://github.com/programatik29/axum-server
pub async fn serve(addr: &SocketAddr, mut app: Router) {
    static IS_FIRST_CALL: AtomicBool = AtomicBool::new(true);
    assert!(IS_FIRST_CALL.load(Ordering::SeqCst), "called twice");
    IS_FIRST_CALL.store(false, Ordering::SeqCst);

    // make the clone() cheaper. https://docs.rs/axum/0.6.1/src/axum/routing/mod.rs.html#538-542
    app = app.with_state(());

    let mut tls_cfg = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(
            vec![Certificate(db_get("ssl_cert").unwrap())],
            PrivateKey(db_get("ssl_key").unwrap()),
        )
        .unwrap();
    // enable http2, needs hyper feature "http2"
    tls_cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    // safety: this fn called only once, so we don't need once_cell::sync::Lazy.
    let tls_acceptor = unsafe {
        static mut TLS_ACCEPTOR: MaybeUninit<TlsAcceptor> = MaybeUninit::uninit();
        TLS_ACCEPTOR.write(TlsAcceptor::from(Arc::new(tls_cfg)));
        TLS_ACCEPTOR.assume_init_ref()
    };
    let protocol = unsafe {
        static mut PROTOCOL: MaybeUninit<Http> = MaybeUninit::uninit();
        PROTOCOL.write(Http::new());
        PROTOCOL.assume_init_ref()
    };

    let mut listener = AddrIncoming::bind(addr).unwrap();

    loop {
        let mut stream = match poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx)).await {
            Some(Ok(v)) => v,
            _ => continue, // ignore error here
        };

        let svc = app.clone();
        tokio::spawn(tokio::time::timeout(TIMEOUT, async move {
            // redirect HTTP to HTTPS
            let mut flag = [0]; // expect 0x16, TLS handshake
            let mut buf = tokio::io::ReadBuf::new(&mut flag);
            poll_fn(|cx| stream.poll_peek(cx, &mut buf)).await.ok();
            if flag[0] != 0x16 {
                stream.write_all(TO_HTTPS_PAGE).await.ok();
                stream.shutdown().await.ok(); // remember to close stream
                return;
            }

            if let Ok(tls_stream) = tls_acceptor.accept(stream).await {
                protocol
                    .serve_connection(tls_stream, svc)
                    // .with_upgrades() // allow WebSocket
                    .await
                    .ok();
            }
        }));
    }
}

// https://nginx.org/en/docs/http/ngx_http_core_module.html#keepalive_timeout
const TIMEOUT: Duration = Duration::from_secs(75);

const TO_HTTPS_PAGE: &[u8] = b"HTTP/1.1 200 OK\r\ncontent-type:text/html\r\n\r\n\
<script>location=location.href.replace(':','s:')</script>\r\n\r\n\0";
