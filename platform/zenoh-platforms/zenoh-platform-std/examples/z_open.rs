#![cfg_attr(
    not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_arch = "wasm32",
    )),
    no_std
)]

use core::str::FromStr;

use static_cell::StaticCell;
use zenoh::{api::session::SessionRunner, EndPoint};
use zenoh_platform_std::PlatformStd;

#[embassy_executor::task]
async fn session_task(mut runner: SessionRunner<'static, PlatformStd>) {
    runner.run().await;
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    log::info!("Starting z_open example...");

    static PLATFORM: StaticCell<PlatformStd> = StaticCell::new();
    let platform = PLATFORM.init(PlatformStd);

    let (mut session, runner) = zenoh::api::session::SingleLinkClientSession::open(
        platform,
        EndPoint::from_str("tcp/127.0.0.1:7447").unwrap(),
    )
    .await
    .unwrap();

    spawner.spawn(session_task(runner)).unwrap();

    loop {
        session.read().await.unwrap();
    }
}
