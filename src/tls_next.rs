use crate::db;
use hyper::server::conn::AddrIncoming;
use std::net::SocketAddr;
use std::sync::Arc;
use tls_listener::TlsListener;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;

fn db_get(k: &str) -> Vec<u8> {
    let result = db!("SELECT v FROM admin WHERE k = ?", [k], (0));
    result.unwrap().pop().unwrap().0
}

pub fn incoming(addr: &SocketAddr) -> TlsListener<AddrIncoming, TlsAcceptor> {
    let cert = Certificate(db_get("ssl_cert"));
    let key = PrivateKey(db_get("ssl_key"));
    let mut server_cfg = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap();
    server_cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    TlsListener::new(
        Arc::new(server_cfg).into(),
        AddrIncoming::bind(&addr).unwrap(),
    )
}
