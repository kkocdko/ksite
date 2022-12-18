//! Protect the process, do auto-restart and more.

use std::env;
use std::fs::File;
use std::future::Future;
use std::io::Write as _;
use std::process::Command;

pub fn launch<F, Fut>(main: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    const BARE_SWITCH: &str = "--bare";
    if env::args_os().any(|v| v == BARE_SWITCH) {
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
    ));
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
        let exit_status = Command::new(env::current_exe().unwrap())
            .arg(BARE_SWITCH)
            .stdout(log_file.try_clone().unwrap())
            .stderr(log_file.try_clone().unwrap())
            .status()
            .unwrap();
        writeln!(&mut log_file, "{exit_status}").unwrap();
    }
}
