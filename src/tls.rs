use crate::db;
use hyper::server::conn::AddrIncoming;
use std::net::SocketAddr;
use tls_listener::TlsListener;
use tokio_native_tls::native_tls::Identity;
use tokio_native_tls::native_tls::TlsAcceptor as TlsAcceptorBuilder;
use tokio_native_tls::TlsAcceptor;

fn db_get(k: &str) -> Vec<u8> {
    let result = db!("SELECT v FROM admin WHERE k = ?", [k], (0));
    result.unwrap().pop().unwrap().0
}

fn tls_acceptor() -> TlsAcceptor {
    let identity = Identity::from_pkcs8(&db_get("ssl_cert"), &db_get("ssl_key")).unwrap();
    let builder = TlsAcceptorBuilder::builder(identity);
    builder.build().unwrap().into()
}

pub fn incoming(addr: &SocketAddr) -> TlsListener<AddrIncoming, TlsAcceptor> {
    TlsListener::new(tls_acceptor(), AddrIncoming::bind(&addr).unwrap())
}
