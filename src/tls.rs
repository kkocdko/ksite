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

pub enum TlsStream {
    Handshaking(tokio_rustls::Accept<AddrStream>),
    Streaming(tokio_rustls::server::TlsStream<AddrStream>),
}

impl TlsStream {
    fn new(stream: AddrStream, cfg: Arc<ServerConfig>) -> Self {
        let accept = tokio_rustls::TlsAcceptor::from(cfg).accept(stream);
        Self::Handshaking(accept)
    }
}

impl AsyncRead for TlsStream {
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

impl AsyncWrite for TlsStream {
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

pub struct TlsAcceptor {
    cfg: Arc<ServerConfig>,
    incoming: AddrIncoming,
}

impl Accept for TlsAcceptor {
    type Conn = TlsStream;
    type Error = io::Error;
    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match ready!(Pin::new(&mut self.incoming).poll_accept(cx)) {
            Some(Ok(mut sock)) => {
                // let flag = ready!(sock.read_i8()).unwrap();
                // let mut flag = [0];
                // match ready!(sock.poll_peek(cx, &mut tokio::io::ReadBuf)) {
                //     Ok(_) => {}
                //     Err(e) => return Poll::Ready(Some(Err(e))),
                // }
                // if flag != [0x16] {
                //     println!("NO TLS");
                //     return Poll::Ready(None);
                // }
                Poll::Ready(Some(Ok(TlsStream::new(sock, self.cfg.clone()))))
            }
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

pub fn incoming(addr: &SocketAddr) -> TlsAcceptor {
    // poem's source code?
    fn db_get(k: &str) -> Vec<u8> {
        let result = db!("SELECT v FROM admin WHERE k = ?", [k], (0));
        result.unwrap().pop().unwrap().0
    }

    let mut cfg = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth() // see warp's source code?
        .with_single_cert(
            vec![Certificate(db_get("ssl_cert"))],
            PrivateKey(db_get("ssl_key")),
        )
        .unwrap();
    // configure ALPN to accept HTTP/2, HTTP/1.1 in that order
    cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    TlsAcceptor {
        cfg: Arc::new(cfg),
        incoming: AddrIncoming::bind(&addr).unwrap(),
    }
}
