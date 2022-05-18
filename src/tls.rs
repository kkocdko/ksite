// thanks to github.com/ctz/hyper-rustls/blob/master/examples/server.rs
use crate::db;
use core::task::{Context, Poll};
use futures_util::ready;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream};
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

pub enum Connection {
    Handshaking(tokio_rustls::Accept<AddrStream>),
    Streaming(tokio_rustls::server::TlsStream<AddrStream>),
}

impl Connection {
    fn new(stream: AddrStream, cfg: Arc<ServerConfig>) -> Self {
        let accept = tokio_rustls::TlsAcceptor::from(cfg).accept(stream);
        Self::Handshaking(accept)
    }
}

impl AsyncRead for Connection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        match &mut *self {
            Self::Handshaking(accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_read(cx, buf);
                    *self = Self::Streaming(stream);
                    result
                }
                Err(e) => Poll::Ready(Err(e)),
            },
            Self::Streaming(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Connection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            Self::Handshaking(accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_write(cx, buf);
                    *self = Self::Streaming(stream);
                    result
                }
                Err(e) => Poll::Ready(Err(e)),
            },
            Self::Streaming(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        match &mut *self {
            Self::Handshaking(_) => Poll::Ready(Ok(())),
            Self::Streaming(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        match &mut *self {
            Self::Handshaking(_) => Poll::Ready(Ok(())),
            Self::Streaming(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

pub struct Acceptor {
    cfg: Arc<ServerConfig>,
    incoming: AddrIncoming,
}

impl Accept for Acceptor {
    type Conn = Connection;
    type Error = io::Error;
    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match ready!(Pin::new(&mut self.incoming).poll_accept(cx)) {
            Some(Ok(mut sock)) => {
                // redirect http to https?
                // let mut flag = [0];
                // ready!(sock.poll_peek(cx, &mut tokio::io::ReadBuf::new(&mut flag)))
                // dbg!(flag == [0x16]);
                Poll::Ready(Some(Ok(Connection::new(sock, self.cfg.clone()))))
            }
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

pub fn incoming(addr: &SocketAddr) -> Acceptor {
    fn db_get(k: &str) -> Vec<u8> {
        let result = db!("SELECT v FROM admin WHERE k = ?", [k], (0));
        result.unwrap().pop().unwrap().0
    }

    let certs = vec![Certificate(db_get("ssl_cert"))];
    let key = PrivateKey(db_get("ssl_key"));
    let mut cfg = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth() // vertify? see warp's source code?
        .with_single_cert(certs, key)
        .unwrap();
    cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Acceptor {
        cfg: Arc::new(cfg),
        incoming: AddrIncoming::bind(&addr).unwrap(),
    }
}
