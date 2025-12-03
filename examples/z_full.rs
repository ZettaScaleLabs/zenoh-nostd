use embassy_time::Timer;

use static_cell::StaticCell;
use zenoh_nostd::{
    api::{PublicResources, *},
    *,
};
use zenoh_std::PlatformStd;

zimport_types!(
    PLATFORM: PlatformStd,
    TX: [u8; 512],
    RX: [u8; 512],

    MAX_KEYEXPR_LEN: 64,
    MAX_PARAMETERS_LEN: 128,
    MAX_PAYLOAD_LEN: 512,

    MAX_QUEUED: 8,
    MAX_CALLBACKS: 8,

    MAX_SUBSCRIBERS: 8,
);

fn callback(sample: &Sample) {
    zenoh_nostd::info!(
        "[Callback] Received sample ('{}': '{}')",
        sample.keyexpr().as_str(),
        ::core::str::from_utf8(sample.payload()).unwrap()
    );
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    env_logger::init();

    let config = Config::new(PlatformStd {}, [0u8; 512], [0u8; 512]);

    static RESOURCES: StaticCell<Resources> = StaticCell::new();
    let mut resources = PublicResources::new();

    let cb1 = resources.subscriber_sync(callback).await.unwrap();

    let session = zenoh_nostd::api::open(
        RESOURCES.init(resources),
        config,
        EndPoint::try_from("tcp/127.0.0.1:7447").unwrap(),
    )
    .await
    .unwrap();

    spawner.spawn(run(session.clone())).unwrap();

    let ke = keyexpr::new("demo/example/**").unwrap();
    // let payload = b"Hello, from no-std!";

    let sub = session.declare_subscriber(ke, cb1).finish().await.unwrap();

    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }

    // loop {
    //     session
    //         .put(ke, payload)
    //         .attachment(b"z_open example")
    //         .encoding(Encoding::bytes())
    //         .finish()
    //         .await
    //         .unwrap();

    //     zenoh_nostd::info!(
    //         "[Put] Sent PUT ('{}': '{}')",
    //         ke.as_str(),
    //         ::core::str::from_utf8(payload).unwrap()
    //     );

    //     Timer::after(embassy_time::Duration::from_secs(1)).await;
    // }
}

#[embassy_executor::task]
async fn run(session: Session<'static>) {
    if let Err(e) = session.run().await {
        zenoh_nostd::error!("Session error: {:?}", e);
    }
}
