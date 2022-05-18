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
    let certs = vec![Certificate(db_get("ssl_cert"))];
    let key = PrivateKey(db_get("ssl_key"));
    let cfg = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();
    TlsListener::new(Arc::new(cfg).into(), AddrIncoming::bind(&addr).unwrap())
}
