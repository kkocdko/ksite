mod auth;
mod database;
mod launcher;
mod ticker;
mod units;
mod utils;
use std::net::SocketAddr;
use std::time::Duration;

// #[global_allocator]
// static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc; // or rpmalloc::RpMalloc

fn main() {
    launcher::launch(run);
    // launcher::launch(bench);
}

async fn run() {
    log!(info: "crate::run");

    // db_upgrade(); // uncomment this if we need to upgrade database

    let server = async {
        let app = axum::Router::new()
            .merge(units::admin::service())
            .merge(units::chat::service())
            .merge(units::copilotgpt::service())
            .merge(units::dav::service())
            .merge(units::info::service())
            .merge(units::magazine::service())
            .merge(units::meet::service())
            // .merge(units::mirror::service())
            // .merge(units::proxy::service())
            .merge(units::qqbot::service())
            .route(
                "/robots.txt",
                axum::routing::MethodRouter::new().get("User-agent: *\nDisallow: /\n"),
            )
            .layer(axum::middleware::from_fn(
                |req: axum::extract::Request, next: axum::middleware::Next| async {
                    log!(trac: "request {:?} {} {} {:?}", req.version(), req.method(), req.uri(), req.headers());
                    next.run(req).await
                },
            ))
            ;
        log!(info: "auth key = {}", auth::auth_key());
        let addr = SocketAddr::from(([0, 0, 0, 0], 9304)); // server address here
        log!(info: "server address = {addr}");
        let tcp_listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let tls_config = {
            use tls_http::*;
            fn get_or_default(k: &str, default: &[u8]) -> Vec<u8> {
                let got = crate::utils::block_on(crate::units::admin::db::get(k.to_owned()));
                got.map(Vec::from).unwrap_or_else(|| {
                    log!(warn: "fallback to default tls cert, ca and key");
                    Vec::from(default)
                })
            }
            mod default_cert {
                include!("tls.defaults.rs"); // for local dev, just ignore cert error and continue
            }
            fn find_subseq<T: PartialEq>(haystack: &[T], needle: &[T]) -> Option<usize> {
                haystack.windows(needle.len()).position(|w| w == needle)
            }
            let ca = CertificateDer::from(get_or_default("tls_ca", default_cert::CA));
            let cert = CertificateDer::from(get_or_default("tls_cert", default_cert::CERT));
            let key = match get_or_default("tls_key", default_cert::KEY) {
                // https://oidref.com/1.2.840.113549.1.1 | https://stackoverflow.com/q/5929050/
                v if find_subseq(&v, &[42, 134, 72, 134, 247, 13, 1]).is_some() => {
                    PrivatePkcs8KeyDer::from(v).into()
                }
                v if find_subseq(&v, &[2, 130, 1, 1, 0]).is_some() => {
                    PrivatePkcs1KeyDer::from(v).into()
                }
                _ => unimplemented!("unknown type of tls_key"),
            };
            let mut tls_config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![cert, ca], key)
                .unwrap();
            tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()]; // HTTP2 needs hyper features = ["http2"]
            tls_config
        };
        tls_http::serve(tcp_listener, app, tls_config).await;
        // axum::serve(tcp_listener, app).await.unwrap();
    };

    let oscillator = async {
        const INTERVAL: Duration = Duration::from_secs(60);
        const TIMEOUT: Duration = Duration::from_secs(45);
        log!(info: "oscillator interval = {INTERVAL:?}, timeout = {TIMEOUT:?}");
        async fn tasks() {
            tokio::join!(
                units::magazine::tick(),
                units::qqbot::tick(),
                units::v2exdaily::tick(),
            );
        }
        let mut interval = tokio::time::interval(INTERVAL);
        loop {
            interval.tick().await;
            care!(tokio::time::timeout(TIMEOUT, tasks()).await).ok();
            // let stamp = httpdate::fmt_http_date(std::time::SystemTime::now());
            // log!("oscillator loop bottom, at {stamp}");
        }
    };

    tokio::join!(server, oscillator);
}

/// Deal with database upgrade.
#[cfg(feature = "db-upgrade")]
fn db_upgrade() {
    const CURRENT_VER: &str = env!("CARGO_PKG_VERSION");
    fn db_set(k: &str, v: &[u8]) {
        db!("REPLACE INTO admin VALUES (?, ?)", [k, v]).unwrap();
    }
    fn db_get(k: &str) -> Option<(Vec<u8>,)> {
        db!("SELECT v FROM admin WHERE k = ?", [k], ^(0)).ok()
    }
    if !matches!(
        db_get("version"),
        Some((v,)) if v == CURRENT_VER.as_bytes()
    ) {
        log!("upgrade database structure to v{CURRENT_VER}");
        db_set("version", CURRENT_VER.as_bytes());
        db!("DROP TABLE health_list").unwrap();
    }
}

/*

mod mono0 {
    use tokio::sync::{mpsc, oneshot};
    pub struct Mono<T: Send + 'static> {
        tx: mpsc::Sender<Box<dyn FnOnce(&mut T) -> () + Send>>,
    }
    impl<T: Send + 'static> Mono<T> {
        pub fn new(mut v: T) -> Self {
            let (tx, mut rx) = mpsc::channel::<Box<dyn FnOnce(&mut T) -> () + Send>>(1);
            std::thread::spawn(move || {
                while let Some(mut f) = rx.blocking_recv() {
                    f(&mut v);
                }
            });
            Self { tx }
        }
        pub async fn call<R: Send + 'static>(&self, f: impl Fn(&mut T) -> R + Send + 'static) -> R {
            let (send, response) = oneshot::channel();
            self.tx
                .send(Box::new(move |s| {
                    // f may be inlined, it's fine
                    send.send(f(s));
                }))
                .await;
            // std::thread::scope(f)
            // tokio::task::spawn_blocking(f)
            return response.await.unwrap();
        }
    }
    async fn main_async() {
        // tokio::task::local
        // let a = tokio::runtime::Handle::current().block_on(async {});
        let mono = Mono::new("abc".to_string());
        let v = mono
            .call(|s| {
                *s += " modified";
                s.to_string()
            })
            .await;
        dbg!(&v);
        // tokio::spawn(async{}).
        // tokio::block_
    }
}

mod mono1 {
    use tokio::sync::{mpsc, oneshot};
    pub struct Mono {
        tx: mpsc::Sender<Box<dyn FnOnce(&mut String) -> () + Send>>,
    }
    impl Mono {
        pub fn new(mut v: String) -> Self {
            let (tx, mut rx) = mpsc::channel::<Box<dyn FnOnce(&mut String) -> () + Send>>(1);
            std::thread::spawn(move || {
                while let Some(mut f) = rx.blocking_recv() {
                    f(&mut v);
                }
            });
            Self { tx }
        }
        pub async fn call<'env>(
            &self,
            f: impl Fn(&mut String) -> String + Send + 'static,
        ) -> String {
            let (send, response) = oneshot::channel();
            self.tx
                .send(Box::new(move |s| {
                    // f may be inlined, it's fine
                    send.send(f(s));
                }))
                .await;
            // std::thread::scope(f)
            // tokio::task::spawn_blocking(f)
            return response.await.unwrap();
        }
    }
}

mod mono2 {
    use tokio::sync::{mpsc, oneshot};
    use tokio::sync::{Mutex, Semaphore};
    pub struct Mono {
        // state: Semaphore,
        // f: Mutex<Box<dyn FnOnce(&mut String) -> () + Send>>,
        // ret: Mutex<String>,
        tx: mpsc::Sender<Box<dyn FnOnce(&mut String) -> () + Send>>,
    }
    impl Mono {
        pub fn new(mut v: String) -> Self {
            let (tx, mut rx) = mpsc::channel::<Box<dyn FnOnce(&mut String) -> () + Send>>(1);
            std::thread::spawn(move || {
                while let Some(mut f) = rx.blocking_recv() {
                    f(&mut v);
                }
            });
            // panic!()
            Self { tx }
        }
        pub async fn call<'env, F>(&self, f: F) -> String
        where
            F: Fn(&mut String) -> String + Send + 'env,
        {
            let (send, response) = oneshot::channel();
            // f(&mut String::new());
            self.tx
                .send(Box::new(move |s| {
                    // f may be inlined, it's fine
                    send.send(f(s));
                }))
                .await;
            // std::thread::scope(f)
            // tokio::task::spawn_blocking(f)
            return response.await.unwrap();
        }
    }
}

async fn bench() {
    // 方案一：tokio::task::spawn_blocking 解决一切
    // 方案二：放在单个 std::thread 里运行，mpsc 或者其他方法传入 Fn 取出 Value

    use crate::utils::LazyLock;
    use mono1::Mono;
    // tokio::task::local
    // let a = tokio::runtime::Handle::current().block_on(async {});
    static MONO: LazyLock<Mono> = LazyLock::new(|| Mono::new("abc".to_string()));
    // async fn append(appended: &str) -> String {
    //     MONO.call(|s| {
    //         *s += appended;
    //         s.to_string()
    //     })
    //     .await
    // }
    // tokio::task::spawn_blocking(f)
    // tokio::spawn(async{}).
    // tokio::block_
}

 */
