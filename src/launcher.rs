//! Protect the process, do auto-restart and more.

use crate::log;
use crate::utils::LazyLock;
use std::env;
use std::fs::File;
use std::future::Future;
use std::io::Write as _;
use std::process::Command;

pub static LOG_FILE: LazyLock<File> = LazyLock::new(|| {
    File::options()
        .append(true)
        .create(true)
        .read(true) // allow admin unit to read
        .open(env::current_exe().unwrap().with_extension("log"))
        .unwrap()
});

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
            .block_on(main());
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
