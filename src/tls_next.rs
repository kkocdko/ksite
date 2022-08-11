// thanks to:
// https://github.com/tokio-rs/axum/blob/main/examples/low-level-rustls
// https://github.com/programatik29/axum-server
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

pub async fn serve(addr: &SocketAddr, mut app: IntoMakeService<Router>) {
    fn db_get(k: &str) -> Vec<u8> {
        let r = db!("SELECT v FROM admin WHERE k = ?", [k], |r| r.get(0));
        r.unwrap().pop().unwrap()
    }

    let certs = vec![Certificate(db_get("ssl_cert"))];
    let key = PrivateKey(db_get("ssl_key"));
    let mut rustls_cfg = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth() // TODO: vertify? see warp's source code?
        .with_single_cert(certs, key)
        .unwrap();
    // HTTP2 needs hyper / axum feature "http2"
    rustls_cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let tls_acceptor = TlsAcceptor::from(Arc::new(rustls_cfg));
    let mut listener = AddrIncoming::bind(addr).unwrap();

    loop {
        let acceptor = tls_acceptor.clone(); // Arc inner
        let mut stream = match poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx)).await {
            Some(Ok(v)) => v,
            _ => continue, // ignore error here
        };

        let mut flag = [0];
        let mut buf = tokio::io::ReadBuf::new(&mut flag);
        poll_fn(|cx| stream.poll_peek(cx, &mut buf)).await.ok();
        // is not tls handshake
        if flag[0] != 0x16 {
            let to_https_page = b"HTTP/1.1 200 OK\r\ncontent-type:text/html\r\n\r\n<script>location=location.href.replace(':','s:')</script>\r\n\r\n\0";
            let _ = stream.write_all(to_https_page).await;
            stream.shutdown().await.ok(); // remember to close stream
            continue;
        }

        // https://docs.rs/hyper/0.14.20/src/hyper/server/server.rs.html#176
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
