use embassy_futures::join::join;
use embassy_time::Timer;

use static_cell::StaticCell;
use zenoh_nostd::*;
use zenoh_std::PlatformStd;

zimport_types!(
    PLATFORM: PlatformStd,
    TX_BUF: [u8; 512],
    RX_BUF: [u8; 512]
);

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    env_logger::init();

    let config = Config {
        platform: PlatformStd {},
        tx_buf: [0u8; 512],
        rx_buf: [0u8; 512],
    };

    static RESOURCES: StaticCell<Resources> = StaticCell::new();
    let session = zenoh_nostd::open(
        RESOURCES.init(Resources::Uninitialized),
        config,
        EndPoint::try_from("tcp/127.0.0.1:7447").unwrap(),
    )
    .await
    .unwrap();

    spawner.spawn(run(session.clone())).unwrap();

    let ke = keyexpr::new("demo/example").unwrap();
    let payload = b"Hello, from no-std!";

    join(session.run(), async {
        loop {
            session.put(ke, payload).await.unwrap();

            zenoh_nostd::info!(
                "[Put] Sent PUT ('{}': '{}')",
                ke.as_str(),
                ::core::str::from_utf8(payload).unwrap()
            );

            Timer::after(embassy_time::Duration::from_secs(1)).await;
        }
    })
    .await;
}

#[embassy_executor::task]
async fn run(session: Session<'static>) {
    if let Err(e) = session.run().await {
        zenoh_nostd::error!("Session error: {:?}", e);
    }
}
