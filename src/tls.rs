// thanks to:
// github.com/ctz/hyper-rustls/blob/master/examples/server.rs
// github.com/seanmonstar/warp/pull/431
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
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

pub enum Connection {
    Handshaking(tokio_rustls::Accept<AddrStream>),
    Streaming(tokio_rustls::server::TlsStream<AddrStream>),
}

impl Connection {
    fn new(stream: AddrStream, cfg: &Arc<ServerConfig>) -> Self {
        let accept = tokio_rustls::TlsAcceptor::from(cfg.clone()).accept(stream);
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
        // redirect http to https
        // use futures_util::FutureExt;
        // use tokio::io::AsyncWriteExt;
        // let mut stream = match ready!(Pin::new(&mut self.incoming).poll_accept(cx)) {
        //     Some(Ok(v)) => v,
        //     _ => return Poll::Ready(None),
        // };
        // let mut flag = [0];
        // let mut readbuf = tokio::io::ReadBuf::new(&mut flag);
        // let mut loop_count = 0;
        // // attacker could make a connection without sending any data
        // while stream.poll_peek(cx, &mut readbuf).is_pending() {
        //     loop_count += 1;
        //     std::thread::sleep(std::time::Duration::from_millis(1));
        // }
        // dbg!(loop_count);
        // if flag[0] != 0x16 {
        //     let to_https_page = b"HTTP/1.1 200 OK\r\ncontent-type:text/html\r\n\r\n<script>location=location.href.replace(':','s:')</script>\r\n\r\n\0";
        //     let _ = stream.write_all(to_https_page).boxed().poll_unpin(cx);
        // }
        // Poll::Ready(Some(Ok(Connection::new(stream, &self.cfg))))

        // https only
        Poll::Ready(match ready!(Pin::new(&mut self.incoming).poll_accept(cx)) {
            Some(Ok(stream)) => Some(Ok(Connection::new(stream, &self.cfg))),
            _ => None,
        })
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
        .with_no_client_auth() // TODO: vertify? see warp's source code?
        .with_single_cert(certs, key)
        .unwrap();
    cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Acceptor {
        cfg: Arc::new(cfg),
        incoming: AddrIncoming::bind(addr).unwrap(),
    }
}
