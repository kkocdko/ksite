use http::uri::Scheme;
use hyper::body::{Body, Incoming};
use hyper::{Request, Response};
use hyper_util::rt::tokio::{TokioExecutor, TokioIo};
use hyper_util::service::TowerToHyperService;
use std::convert::Infallible;
use std::future::poll_fn;
use std::sync::Arc;
use tokio::io;
use tokio::io::AsyncWriteExt;
pub use tokio_rustls::rustls::pki_types::*;
pub use tokio_rustls::rustls::{ClientConfig, RootCertStore, ServerConfig};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tower_service::Service;

#[derive(Debug)]
pub enum ClientError {
    Connect(io::Error),
    Hyper(hyper::Error),
}

impl From<hyper::Error> for ClientError {
    fn from(value: hyper::Error) -> Self {
        Self::Hyper(value)
    }
}

impl From<io::Error> for ClientError {
    fn from(value: io::Error) -> Self {
        Self::Connect(value)
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ClientError {}

/// Simple http/https client use hyper and rustls.
///
/// Unlike the full-featured reqwest crate, this one doesn't have some advance function like connection pool, redirection follow, auto cookie and others. You have to implement them manually if you want.
///
/// # Examples
///
/// Create a default client (with CA list from webpki-roots) and fetch something:
///
/// ```
/// ```
pub struct Client(pub Arc<ClientConfig>);

impl Client {
    pub fn new_without_verify() -> Self {
        //         use tokio_rustls::rustls::client::danger::ServerCertVerifier;
        //         struct EmptyVerifier;
        //         impl ServerCertVerifier for EmptyVerifier{
        // fn supported_verify_schemes(&self) -> Vec<tokio_rustls::rustls::SignatureScheme> {

        // }
        // fn
        //         }
        //         let tls_config = ClientConfig::builder().dangerous().with_custom_certificate_verifier(verifier)
        //         Self(Arc::new(tls_config))
        unimplemented!()
    }

    pub fn new_with_webpki_roots() -> Self {
        // let mut ca = Vec::new();
        // for entry in std::fs::read_dir("/etc/ssl/certs").unwrap() {
        //     let entry = entry.unwrap();
        //     if !entry.metadata().unwrap().is_file() {
        //         continue;
        //     }
        //     let s = std::fs::read_to_string(entry.path()).unwrap();
        //     let mut der_base64 = String::new();
        //     for line in s.split('\n').skip(1) {
        //         if line.starts_with("-----") {
        //             break;
        //             // TODO?
        //         }
        //         der_base64 += line;
        //     }
        //     {
        //         use base64::{engine::general_purpose, Engine as _};
        //         let bytes = general_purpose::STANDARD.decode(der_base64).unwrap();
        //         ca.push(bytes);
        //     }
        // }
        // let config = TlsConfig::new_client(Some(ca));
        // let config = TlsConfig::new_client(None);
        let mut tls_config = ClientConfig::builder()
            .with_root_certificates(RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
            })
            .with_no_client_auth();
        tls_config.alpn_protocols = vec![b"http/1.1".to_vec()];
        tls_config.enable_sni = false;
        Self(Arc::new(tls_config))
    }

    pub async fn fetch<B>(
        &self,
        req: Request<B>,
        resolved: Option<String>,
    ) -> Result<Response<Incoming>, ClientError>
    where
        B: Body + 'static + Send,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let uri = req.uri();
        let scheme = uri.scheme();
        let host = uri.host().unwrap();
        let mut host_and_port = String::with_capacity(host.len() + 8);
        host_and_port += host;
        host_and_port += ":";
        match uri.port_u16() {
            Some(port) => {
                use std::fmt::Write as _;
                write!(&mut host_and_port, "{port}").unwrap();
            }
            None if scheme == Some(&Scheme::HTTP) => host_and_port += "80",
            None if scheme == Some(&Scheme::HTTPS) => host_and_port += "443",
            None => panic!("unsupported scheme"),
        };
        let tcp_stream = tokio::net::TcpStream::connect(resolved.unwrap_or(host_and_port)).await?;

        if scheme == Some(&Scheme::HTTP) {
            let io = TokioIo::new(tcp_stream);
            let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?; // only http1 currently
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    println!("Connection failed: {:?}", err);
                }
            });
            return Ok(sender.send_request(req).await?);
        }

        if uri.scheme() == Some(&Scheme::HTTPS) {
            let tls_connector = TlsConnector::from(self.0.clone());
            let tls_stream = tls_connector
                .connect(ServerName::try_from(host.to_string()).unwrap(), tcp_stream)
                .await?;
            let io = TokioIo::new(tls_stream);
            let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    println!("Connection failed: {:?}", err);
                }
            });
            return Ok(sender.send_request(req).await?);
        }

        panic!("unsupported scheme")
    }
}

pub async fn serve<S, B>(
    tcp_listener: tokio::net::TcpListener,
    service: S,
    tls_config: ServerConfig,
) where
    B: Body + Send + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S: Service<Request<Incoming>, Response = Response<B>, Error = Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send,
{
    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));
    let service = TowerToHyperService::new(service);
    loop {
        let service = service.clone();
        let (mut tcp_stream, _socket_addr) = match tcp_listener.accept().await {
            Ok(v) => v,
            _ => continue, // ignore error here?
        };
        // dbg!(socket_addr);
        let tls_acceptor = tls_acceptor.clone();
        tokio::spawn(async move {
            // redirect HTTP to HTTPS
            let mut flag = [0]; // expect 0x16, TLS handshake
            let mut buf = tokio::io::ReadBuf::new(&mut flag);
            poll_fn(|cx| tcp_stream.poll_peek(cx, &mut buf)).await.ok();
            if flag[0] != 0x16 {
                tcp_stream.write_all(TO_HTTPS_PAGE).await.ok();
                tcp_stream.shutdown().await.ok(); // remember to close stream
                return;
            }
            let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                Ok(v) => v,
                Err(_e) => return,
            };
            let io = TokioIo::new(tls_stream);
            hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection(io, service)
                // .serve_connection_with_upgrades(io, service)
                .await
                .ok();
        });
    }
}

// TODO: Use HSTS?
const TO_HTTPS_PAGE: &[u8] = b"HTTP/1.1 200 OK\r\ncontent-type:text/html\r\n\r\n<script>location=location.href.replace(':','s:')</script><h2>Please visit this site using HTTPS.</h2>\r\n\r\n\0";
