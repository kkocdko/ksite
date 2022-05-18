// use futures_util::{
//     stream::{BoxStream, Chain, Pending},
//     Stream, StreamExt, TryFutureExt,
// };
// // use http::uri::Scheme;
// use tokio::io::{Error as IoError, ErrorKind, Result as IoResult};
use crate::db;
use core::task::{Context, Poll};
use futures_util::ready;
use futures_util::stream::StreamExt;
use hyper::server::accept;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server};
use std::convert::Infallible;
use std::fs;
use std::future::ready;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{Error as IoError, ErrorKind, Result as IoResult};
use tokio_native_tls::{
    native_tls::Identity, TlsAcceptor as NativeTlsAcceptor, TlsStream as NativeTlsStream,
};
fn db_get(k: &str) -> Vec<u8> {
    let result = db!("SELECT v FROM admin WHERE k = ?", [k], (0));
    result.unwrap().pop().unwrap().0
}

pub fn tls_acceptor() -> tokio_native_tls::TlsAcceptor {
    use tokio_native_tls::native_tls::{Identity, TlsAcceptor};

    // let mut cert_file = fs::File::open("cert.pem").unwrap();
    // let mut certs = vec![];
    // cert_file.read_to_end(&mut certs).unwrap();
    // let mut key_file = fs::File::open("key.pem").unwrap();
    // let mut key = vec![];
    // key_file.read_to_end(&mut key).unwrap();

    // let identity = Identity::from_pkcs8(&certs, &key).unwrap();
    let identity = Identity::from_pkcs8(&db_get("ssl_cert"), &db_get("ssl_key")).unwrap();
    TlsAcceptor::builder(identity).build().unwrap().into()
}

pub fn incoming(
    addr: &SocketAddr,
) -> tls_listener::TlsListener<AddrIncoming, tokio_native_tls::TlsAcceptor> {
    use hyper::server::accept;
    tls_listener::TlsListener::new(tls_acceptor(), AddrIncoming::bind(&addr).unwrap())
}
// /// Native TLS Config.
// pub struct TlsConfig {
//     cert: Vec<u8>,
//     key: Vec<u8>,
// }

// impl TlsConfig {
//     fn create_acceptor(&self) -> tokio_native_tls::native_tls::TlsAcceptor {
//         let identity = Identity::from_pkcs8(&self.cert, &self.key).unwrap();
//         tokio_native_tls::native_tls::TlsAcceptor::new(identity).unwrap()
//     }
// }

// enum TlsStream {
//     Handshaking(tokio_native_tls::TlsAcceptor),
//     Streaming(tokio_native_tls::TlsStream<AddrStream>),
// }

// impl TlsStream {
//     fn new(stream: AddrStream, cfg: Arc<TlsConfig>) -> Self {
//         let accept = cfg.create_acceptor().accept(stream);
//         Self::Handshaking(accept)
//     }
// }

// pub struct TlsAcceptor {
//     cfg: Arc<TlsConfig>,
//     incoming: AddrIncoming,
// }

// impl Accept for TlsAcceptor {
//     type Conn = TlsStream;
//     type Error = io::Error;
//     fn poll_accept(
//         mut self: Pin<&mut Self>,
//         cx: &mut Context,
//     ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
//         match ready!(Pin::new(&mut self.incoming).poll_accept(cx)) {
//             Some(Ok(mut sock)) => Poll::Ready(Some(Ok(TlsStream::new(sock, self.cfg.clone())))),
//             Some(Err(e)) => Poll::Ready(Some(Err(e))),
//             None => Poll::Ready(None),
//         }
//     }
// }
