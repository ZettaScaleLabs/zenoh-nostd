use core::str::FromStr;

use static_cell::StaticCell;
use zenoh::{api::session::SessionRunner, keyexpr, EndPoint};
use zenoh_platform_std::PlatformStd;

#[embassy_executor::task]
async fn session_task(mut runner: SessionRunner<'static, zenoh_platform_std::PlatformStd>) {
    runner.run().await;
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    log::info!("Starting z_put example...");

    static PLATFORM: StaticCell<PlatformStd> = StaticCell::new();
    let platform = PLATFORM.init(PlatformStd);

    let (mut session, runner) = zenoh::api::session::SingleLinkClientSession::open(
        platform,
        EndPoint::from_str("tcp/127.0.0.1:7447").unwrap(),
    )
    .await
    .unwrap();

    spawner.spawn(session_task(runner)).unwrap();

    let ke: &'static keyexpr = "demo/example".try_into().unwrap();
    let payload = b"Hello, world!";

    loop {
        session.try_read().unwrap();

        session.put(ke, payload).await.unwrap();

        log::info!(
            "[Publisher] Sent PUT ('{}': '{}')",
            ke.as_str(),
            core::str::from_utf8(payload).unwrap()
        );

        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
