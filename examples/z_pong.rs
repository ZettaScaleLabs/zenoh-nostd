use zenoh_nostd::{EndPoint, PlatformStd, keyexpr, zsubscriber};

const CONNECT: Option<&str> = option_env!("CONNECT");

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_pong example");

    let mut session = zenoh_nostd::open!(
        zenoh_nostd::zconfig!(
                PlatformStd: (spawner, PlatformStd {}),
                TX: 512,
                RX: 512,
                MAX_SUBSCRIBERS: 2
        ),
        EndPoint::try_from(CONNECT.unwrap_or("tcp/127.0.0.1:7447")).unwrap()
    )
    .unwrap();

    let ke_pong: &'static keyexpr = "test/pong".try_into().unwrap();
    let ke_ping: &'static keyexpr = "test/ping".try_into().unwrap();

    let sub = session
        .declare_subscriber(
            ke_ping,
            zsubscriber!(QUEUE_SIZE: 8, MAX_KEYEXPR: 32, MAX_PAYLOAD: 128),
        )
        .await
        .unwrap();

    while let Ok(sample) = sub.recv().await {
        session.put(ke_pong, sample.payload()).await.unwrap();
    }
}
