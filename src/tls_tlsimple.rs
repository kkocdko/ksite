//! TLS & HTTPS support for the server.

use crate::care;
use crate::units::admin::db_get;
use crate::utils::OptionResult;
use hyper::server::conn::Http;
use std::future::poll_fn;
use std::mem::{self, size_of};
use std::mem::{ManuallyDrop, MaybeUninit};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tlsimple::{TlsConfig, TlsStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

    static IS_FIRST_CALL: AtomicBool = AtomicBool::new(true);
    assert!(IS_FIRST_CALL.swap(false, Ordering::SeqCst), "called twice");

    let tls_cert_der = care!(db_get("ssl_cert").e()).unwrap_or_else(|_| default_cert::CERT.into());
    let tls_key_der = care!(db_get("ssl_key").e()).unwrap_or_else(|_| default_cert::KEY.into());
    let tls_config = TlsConfig::new_server(&tls_cert_der, &tls_key_der, tlsimple::alpn::H1);

    let protocol = ManuallyDrop::new(Box::new(Http::new()));
    let protocol: &Http = unsafe { &*(protocol.as_ref() as *const _) };

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // event loop
    // let mut tls_config_bak: [u8; size_of::<TlsConfig>()] =
    //     unsafe { std::mem::transmute_copy(&(**tls_config)) };
    loop {
        // let mut tls_config_bak_next: [u8; size_of::<TlsConfig>()] =
        //     unsafe { std::mem::transmute_copy(&(**tls_config)) };
        // if tls_config_bak != tls_config_bak_next {
        //     println!(">>> tls_config_bak != tls_config_bak_next");
        // }

        let (mut stream, socket_addr) = match listener.accept().await {
            Ok(v) => v,
            _ => continue, // ignore error here?
        };
        dbg!(socket_addr);
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
            // tls_stream.accept();
            protocol
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
