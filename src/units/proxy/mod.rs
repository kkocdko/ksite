//! Primitive proxy over http(s).
//!
//! 1. Inline Proxy Mode: view web page in browser directly, for simple usage.
//! 2. Shadowsocks Mode: simple shadowsocks proxy implement.
//! 3. UDP 53 Mode: through client's UDP 53 port.

use crate::utils::OptionResult;
use crate::{care, include_src, log};
use axum::body::{Body, Bytes, HttpBody};
use axum::extract::FromRequest;
use axum::http::header::{HeaderMap, CONTENT_LENGTH, HOST};
use axum::http::header::{HeaderName, HeaderValue};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, EXPIRES, REFRESH};
use axum::http::StatusCode;
use axum::http::{Request, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{MethodFilter, MethodRouter, Router};
use std::any::{Any, TypeId};
use std::future::poll_fn;
use std::mem;
use std::pin::Pin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

// https://jimages.net/archives/269
// https://github.com/dizda/fast-socks5

async fn inline_proxy_handler(mut req: Request<Body>) -> Result<Response<Body>, &'static str> {
    let e = "invalid_request";
    let uri = req.uri_mut();
    let uri: Uri = uri.query().e().or(Err(e))?.try_into().or(Err(e))?;
    *req.uri_mut() = uri;
    // let mut rep = fetch(req).await.or(Err("fetch_failed"))?;
    // view-source:https://127.0.0.1:9304/proxy/inline?https://www.bing.com/
    // TODO: CSP?
    // Ok(rep)
    unimplemented!()
}

// if the server requires it to serve the proper certificate.

// this is a HTTP CONNECT proxy
async fn http_proxy_handler(mut req: Request<Body>, sni: bool) -> Response {
    println!("req: {:?}", req);
    if req.method() != axum::http::Method::CONNECT {
        return (StatusCode::BAD_REQUEST, "").into_response();
    }
    let addr = req.uri().authority().map(|auth| auth.to_string()).unwrap();
    // let u=tls_http::upgrade::on(req).await;
    dbg!(2);
    tokio::task::spawn(async move {
        match tls_http::upgrade::on(req).await {
            Ok(mut upgraded) => {
                let mut tcp_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
                let mut upgraded = tls_http::TokioIo::new(upgraded);

                // Proxying data
                let (from_client, from_server) =
                    tokio::io::copy_bidirectional(&mut upgraded, &mut tcp_stream)
                        .await
                        .unwrap();

                // Print message when done
                println!(
                    "client wrote {} bytes and received {} bytes",
                    from_client, from_server
                );
            }
            e => {
                e.unwrap();
                // log!(erro: "{e}");
            }
        }
    });

    Response::builder().status(200).body(Body::empty()).unwrap()
    // Response::new(Body::empty())

    // Ok(Response::new(Body::empty()))
    // let addr = req.headers().get(HOST).unwrap(); // TODO: req.uri().authority().map(|auth| auth.to_string())
    // let mut body = req.into_body();
    // let (tx, rx) = mpsc::channel::<std::io::Result<Bytes>>(16);
    // tokio::spawn(async move {
    //     // let tcp_stream = tokio::net::TcpStream::connect(addr.to_str().unwrap())
    //     let mut tcp_stream = tokio::net::TcpStream::connect("36.155.160.221:443")
    //         .await
    //         .unwrap();
    //     let (mut read, mut write) = tokio::io::split(tcp_stream);

    //     let f2 = async {
    //         loop {
    //             let mut buf = Vec::new();
    //             care!(read.read_buf(&mut buf).await);
    //             dbg!(buf.len());
    //             care!(tx.send(Ok(buf.into())).await);
    //         }
    //     };
    //     // tokio_util::io::InspectWriter::new(writer, f)
    //     // tokio::io::copy(&mut tcp_stream, &mut tx).await;
    //     let f1 = async {
    //         while let Some(result) = poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await {
    //             match result {
    //                 Ok(frame) if !frame.is_data() => {}
    //                 Ok(frame) => {
    //                     let data = frame.into_data().unwrap();
    //                     dbg!(data.len());
    //                     write.write(&data).await.unwrap();
    //                 }
    //                 Err(e) => {
    //                     log!(erro: "{e:?}");
    //                     return;
    //                 }
    //             }
    //         }
    //     };
    //     tokio::join!(f1, f2);
    // });
    // Response::builder()
    //     .status(200)
    //     .body(axum::body::Body::from_stream(
    //         tokio_stream::wrappers::ReceiverStream::new(rx),
    //     ))
    //     .unwrap()
}

// https://github.com/hyperium/hyper/issues/1884#issuecomment-565557580

pub fn service() -> Router {
    Router::new()
        .route(
            "/proxy", // home page
            MethodRouter::new().get(|| async { Html((include_src!("page.html") as [_; 1])[0]) }),
        )
        .fallback(MethodRouter::new().fallback(|req| http_proxy_handler(req, true)))
        .route(
            "/proxy/http-strip-sni",
            MethodRouter::new().fallback(|req| http_proxy_handler(req, false)),
        )
        .route(
            "/proxy/inline", // inline proxy
            MethodRouter::new().fallback(inline_proxy_handler),
        )
        .route(
            "/proxy/sw.js", // inline proxy
            MethodRouter::new().get(|| async {
                (
                    [
                        (
                            CONTENT_TYPE,
                            HeaderValue::from_static("application/javascript"),
                        ),
                        (CACHE_CONTROL, HeaderValue::from_static("no-store")),
                        // (CACHE_CONTROL, HeaderValue::from_static("max-age=600")),
                    ],
                    (include_src!("sw.js") as [_; 1])[0],
                )
            }),
        )
}
