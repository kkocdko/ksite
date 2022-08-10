// thanks to:
// https://github.com/tokio-rs/axum/blob/main/examples/low-level-rustls
use crate::db;
use axum::routing::IntoMakeService;
use axum::{Router, Server};
use futures_util::future::poll_fn;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream, Http};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tower::MakeService;

pub async fn run(addr: &SocketAddr, mut app: IntoMakeService<Router>) {
    let rustls_config = {
        fn db_get(k: &str) -> Vec<u8> {
            let r = db!("SELECT v FROM admin WHERE k = ?", [k], |r| r.get(0));
            r.unwrap().pop().unwrap()
        }

        let certs = vec![Certificate(db_get("ssl_cert"))];
        let key = PrivateKey(db_get("ssl_key"));
        let mut cfg = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth() // TODO: vertify? see warp's source code?
            .with_single_cert(certs, key)
            .unwrap();
        // HTTP2 needs hyper / axum feature "http2"
        cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        Arc::new(cfg)
    };

    let acceptor = TlsAcceptor::from(rustls_config);
    let listener = TcpListener::bind(addr).await.unwrap();
    let mut listener = AddrIncoming::from_listener(listener).unwrap();

    let mut i: u128 = 0;
    loop {
        if i % 100 == 0 {
            dbg!(&i);
        }
        i += 1;
        let stream = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx))
            .await
            .unwrap()
            .unwrap();

        let acceptor = acceptor.clone(); // Arc inner
        let app = app.make_service(&stream).await.unwrap();

        tokio::spawn(async move {
            if let Ok(stream) = acceptor.accept(stream).await {
                let _ = Http::new().serve_connection(stream, app).await;
            }
        });
    }
}
