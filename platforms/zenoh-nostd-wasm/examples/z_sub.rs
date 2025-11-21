use {
    embassy_executor::Spawner,
    zenoh_nostd::{
        EndPoint, ZResult, ZSample, ZSubscriber, keyexpr, platform::Platform, zsubscriber,
    },
    zenoh_nostd_wasm::PlatformWasm,
};

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

async fn entry(spawner: embassy_executor::Spawner) -> ZResult<()> {
    zenoh_nostd::info!("zenoh-nostd z_sub example");

    let config = zenoh_nostd::zconfig!(
            PlatformWasm: (spawner, PlatformWasm {}),
            TX: 1024,
            RX: 1024,
            MAX_SUBSCRIBERS: 2
    );

    let mut session = zenoh_nostd::open!(
        config,
        EndPoint::try_from(CONNECT.unwrap_or("ws/127.0.0.1:7446"))?
    );

    let ke = keyexpr::new("demo/example").expect("Failed to create key expression");

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
async fn main(spawner: Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh_nostd::error!("Error in main: {:?}", e);
    }

    zenoh_nostd::info!("Exiting main");
}
