use core::str::FromStr;

use embassy_futures::select::select;
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

    log::info!("Starting z_sub example...");

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
    let mut sub = session.declare_subscriber(ke).await.unwrap();

    loop {
        match select(
            embassy_time::Timer::after(embassy_time::Duration::from_secs(1)),
            sub.recv(),
        )
        .await
        {
            embassy_futures::select::Either::First(_) => {
                if let Err(e) = session.try_read().await {
                    log::error!("[Session] Error during read: {e}");
                }
            }
            embassy_futures::select::Either::Second(sample) => {
                log::info!(
                    "[Subscriber] Received Sample ('{}': '{:?}')",
                    ke.as_str(),
                    core::str::from_utf8(sample.as_slice())
                );
            }
        }
    }
}
