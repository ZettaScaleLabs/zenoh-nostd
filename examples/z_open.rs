use embassy_time::Timer;

use static_cell::StaticCell;
use zenoh_nostd::{api::*, *};
use zenoh_std::PlatformStd;

zimport_types!(
    PLATFORM: PlatformStd,
    TX: [u8; 512],
    RX: [u8; 512]
);

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    env_logger::init();

    let config = Config::new(PlatformStd {}, [0u8; 512], [0u8; 512]);

    static RESOURCES: StaticCell<Resources> = StaticCell::new();
    let session = zenoh_nostd::api::open(
        RESOURCES.init(Resources::new()),
        config,
        EndPoint::try_from("tcp/127.0.0.1:7447").unwrap(),
    )
    .await
    .unwrap();

    spawner.spawn(run(session.clone())).unwrap();

    let ke = keyexpr::new("demo/example").unwrap();
    let payload = b"Hello, from no-std!";

    loop {
        session
            .put(ke, payload)
            .attachment(b"z_open example")
            .encoding(Encoding::bytes())
            .finish()
            .await
            .unwrap();

        zenoh_nostd::info!(
            "[Put] Sent PUT ('{}': '{}')",
            ke.as_str(),
            ::core::str::from_utf8(payload).unwrap()
        );

        Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn run(session: Session<'static>) {
    if let Err(e) = session.run().await {
        zenoh_nostd::error!("Session error: {:?}", e);
    }
}
