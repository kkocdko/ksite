//! Protect the process, do auto-restart and more.

use std::env;
use std::fs::File;
use std::future::Future;
use std::io::Write as _;
use std::process::Command;

#[inline(always)]
pub fn launch<F, Fut>(main: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    const WRAPPED_FLAG: &str = "KSITE_WRAPPED";
    if env::var(WRAPPED_FLAG).is_ok() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(main());
        return;
    }
    println!(concat!(
        env!("CARGO_PKG_NAME"),
        " v",
        env!("CARGO_PKG_VERSION"),
        " with launcher"
    ),);
    let mut log_file = File::options()
        .append(true)
        .create(true)
        .open(env::current_exe().unwrap().with_extension("log"))
        .unwrap();
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
        let mut child = Command::new(env::current_exe().unwrap())
            .env(WRAPPED_FLAG, "1")
            .stdout(log_file.try_clone().unwrap())
            .stderr(log_file.try_clone().unwrap())
            .spawn()
            .unwrap();
        let exit_status = child.wait().unwrap();
        writeln!(&mut log_file, "{exit_status}").unwrap();
    }
}
