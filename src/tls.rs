//! TLS & HTTPS support for the server.

use crate::log;
use crate::units::admin::db as db_admin;
use hyper::server::conn::Http;
use std::future::poll_fn;
use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::Duration;
use tlsimple::{alpn, TlsConfig, TlsStream};
use tokio::io::AsyncWriteExt;

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
pub async fn serve(addr: &SocketAddr, svc: axum::Router) {
    let svc = svc.with_state(()); // make the clone() cheaper. https://docs.rs/axum/0.6.1/src/axum/routing/mod.rs.html#538-542

    fn get_with_warn(k: &str, default: &[u8]) -> Vec<u8> {
        db_admin::get(k).unwrap_or_else(|| {
            log!(WARN: "using default cert and key");
            Vec::from(default)
        })
    }
    let tls_cert_der = get_with_warn("ssl_cert", default_cert::CERT);
    let tls_key_der = get_with_warn("ssl_key", default_cert::KEY);
    let tls_config = TlsConfig::new_server(tls_cert_der, tls_key_der, Some(alpn::H2H1));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    fn protocol_get() -> &'static Http {
        static PROTOCOL: OnceLock<Http> = OnceLock::new();
        PROTOCOL.get_or_init(|| {
            let protocol = Http::new();
            // protocol.http1_keep_alive(false);
            protocol
        })
    }

    loop {
        let (mut stream, _socket_addr) = match listener.accept().await {
            Ok(v) => v,
            _ => continue, // ignore error here?
        };
        // dbg!(socket_addr);
        let svc = svc.clone();
        let tls_config = tls_config.clone();
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
            let tls_stream = TlsStream::new_async(tls_config, &mut stream);
            protocol_get()
                .serve_connection(tls_stream, svc)
                // .with_upgrades() // allow WebSocket
                .await
                .ok();
        }));
    }
}

// https://nginx.org/en/docs/http/ngx_http_core_module.html#keepalive_timeout
const TIMEOUT: Duration = Duration::from_secs(75);

const TO_HTTPS_PAGE: &[u8] = b"HTTP/1.1 200 OK\r\ncontent-type:text/html\r\n\r\n\
<script>location=location.href.replace(':','s:')</script>\r\n\r\n\0";

mod default_cert {
    include!("tls.defaults.rs");
}
