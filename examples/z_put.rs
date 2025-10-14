use zenoh_nostd::{
    keyexpr::borrowed::keyexpr, platform::platform_std::PlatformStd,
    protocol::core::endpoint::EndPoint,
};

const CONNECT: Option<&str> = option_env!("CONNECT");

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_put example");

    let mut session = zenoh_nostd::open!(
        zenoh_nostd::zconfig!(
                PlatformStd: (spawner, PlatformStd {}),
                TX: 512,
                RX: 512,
                SUBSCRIBERS: 2
        ),
        EndPoint::try_from(CONNECT.unwrap_or("tcp/127.0.0.1:7447")).unwrap()
    )
    .unwrap();

    let ke: &'static keyexpr = "demo/example".try_into().unwrap();
    let payload = b"Hello, from std!";

    loop {
        session.put(ke, payload).await.unwrap();

        zenoh_nostd::info!(
            "[Publisher] Sent PUT ('{}': '{}')",
            ke.as_str(),
            core::str::from_utf8(payload).unwrap()
        );

        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
