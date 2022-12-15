//! Protect the process, do auto-restart and more.

use std::env;
use std::fs::File;
use std::future::Future;
use std::io::{BufRead, BufReader};
use std::process::Command;
use std::time::UNIX_EPOCH;

#[inline(always)]
pub fn launch<F, Fut>(main: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let ans = run().await;
            dbg!(ans);
        });
    return;
    const WRAPPED_FLAG: &str = "KSITE_WRAPPED";
    if env::var(WRAPPED_FLAG).is_ok() {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(main());
        return;
    }
    loop {
        let exe_path = env::current_exe().unwrap();
        let mut log_file = File::options()
            .append(true)
            .create(true)
            .open(exe_path.with_extension("log"))
            .unwrap();
        let mut child = Command::new(exe_path)
            .env(WRAPPED_FLAG, "1")
            // .stdout(cfg)
            .spawn()
            .unwrap();
        // log_file.clon
        // child.stdout
        // child.std
        child.wait().unwrap();
    }
}

use std::time::Duration;

async fn get_a1() -> Option<i32> {
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("get_a1 before return");
    Some(1)
}

async fn get_a2() -> Option<i32> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("get_a2 before return");
    Some(2)
}

async fn run() -> Option<i32> {
    let mut a2v = None;
    let result = tokio::try_join! {
        async{
            if let Some(v) = get_a1().await {
                return Err(v);
            }
            Ok(())
        },
        async {
            a2v = get_a2().await;
            Ok(())
        }
    };
    if let Err(v) = result {
        return Some(v);
    }
    return a2v;
}
