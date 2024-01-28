//! Protect the process, do auto-restart and more.

use crate::log;
use crate::utils::LazyLock;
use std::env;
use std::fs::File;
use std::future::Future;
use std::io::Write as _;
use std::process::Command;
use std::sync::mpsc::{Receiver as MpscReceiver, SyncSender as MpscSyncSender};
use std::sync::{Arc, Barrier, Mutex};

pub static LOG_FILE: LazyLock<File> = LazyLock::new(|| {
    File::options()
        .append(true)
        .create(true)
        .read(true) // allow admin unit to read
        .open(env::current_exe().unwrap().with_extension("log"))
        .unwrap()
});

#[allow(clippy::type_complexity)]
pub static BLOCK_ON: LazyLock<(
    MpscSyncSender<Box<dyn Future<Output = ()> + Send>>,
    Mutex<Option<MpscReceiver<Box<dyn Future<Output = ()> + Send>>>>,
)> = LazyLock::new(|| {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    (tx, Mutex::new(Some(rx)))
});

pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    // https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/barbara_bridges_sync_and_async.html#is-there-any-way-to-have-kept-aggregate-as-a-synchronous-function
    let m = Arc::new(Mutex::new(None));
    let m1 = Arc::clone(&m);
    let b = Arc::new(Barrier::new(2));
    let b1 = Arc::clone(&b);
    let _ = BLOCK_ON.0.send(Box::new(async move {
        *m1.lock().unwrap() = Some(future.await);
        b1.wait();
    }));
    b.wait();
    let mut m = m.lock().unwrap();
    m.take().unwrap()
}

pub fn launch<F, Fut>(main: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    const BARE_SWITCH: &str = "--bare";
    if env::args_os().any(|v| v == BARE_SWITCH) {
        // tokio::runtime::Builder::new_current_thread()
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let join_handle = tokio::task::spawn_blocking(|| {
                    let mut rx = BLOCK_ON.1.lock().unwrap();
                    let rx = rx.take().expect("get block_on receiver failed");
                    while let Ok(future) = rx.recv() {
                        tokio::runtime::Handle::current().block_on(Box::into_pin(future));
                    }
                });
                let _ = tokio::join!(join_handle, main());
            });
        return;
    }
    log!(concat!(
        env!("CARGO_PKG_NAME"),
        " v",
        env!("CARGO_PKG_VERSION"),
    ));
    // thread::spawn(|| loop {
    //     let mut buf = [0];
    //     io::stdin().read(&mut buf).unwrap();
    //     if buf[0] != b'\n' {
    //         continue;
    //     }
    //     // TODO: show the lastest log content here?
    //     // tail -n16 ksite.log
    // });
    loop {
        let exit_status = Command::new(env::current_exe().unwrap())
            .arg(BARE_SWITCH)
            .stdout(LOG_FILE.try_clone().unwrap())
            .stderr(LOG_FILE.try_clone().unwrap())
            .status()
            .unwrap();
        writeln!(&mut LOG_FILE.try_clone().unwrap(), "{exit_status}").unwrap();
    }
}
