use zenoh_nostd::{EndPoint, ZSample, ZSubscriber, keyexpr, zsubscriber};
use zenoh_std::PlatformStd;

const CONNECT: Option<&str> = option_env!("CONNECT");

fn callback_1(sample: &ZSample) {
    zenoh_nostd::info!(
        "[Subscription Sync] Received Sample ('{}': '{:?}')",
        sample.keyexpr().as_str(),
        core::str::from_utf8(sample.payload()).unwrap()
    );
}

#[embassy_executor::task]
async fn callback_2(subscriber: ZSubscriber<32, 128>) {
    while let Ok(sample) = subscriber.recv().await {
        zenoh_nostd::info!(
            "[Subscription Async] Received Sample ('{}': '{:?}')",
            sample.keyexpr().as_str(),
            core::str::from_utf8(sample.payload()).unwrap()
        );
    }
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_sub example");

    let mut session = zenoh_nostd::open!(
        zenoh_nostd::zconfig!(
                PlatformStd: (spawner, PlatformStd {}),
                TX: 512,
                RX: 512,
                MAX_SUBSCRIBERS: 2
        ),
        EndPoint::try_from(CONNECT.unwrap_or("tcp/127.0.0.1:7447")).unwrap()
    );

    let ke = keyexpr::new("demo/example/**").unwrap();

    let _sync_sub = session
        .declare_subscriber(ke, zsubscriber!(callback_1))
        .await
        .unwrap();

    let async_sub = session
        .declare_subscriber(
            ke,
            zsubscriber!(QUEUE_SIZE: 8, MAX_KEYEXPR: 32, MAX_PAYLOAD: 128),
        )
        .await
        .unwrap();

    spawner.spawn(callback_2(async_sub)).unwrap();

    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh_nostd::error!("Error in main: {:?}", e);
    }
}
