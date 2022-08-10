// thanks to:
// https://github.com/tokio-rs/axum/blob/main/examples/low-level-rustls
use crate::db;
use futures_util::ready;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream};
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
