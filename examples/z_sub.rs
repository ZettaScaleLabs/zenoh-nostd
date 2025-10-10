use embassy_executor::Spawner;

use zenoh_nostd::{
    api::sample::ZSample, keyexpr::borrowed::keyexpr, platform::platform_std::PlatformStd,
    protocol::core::endpoint::EndPoint, zcallback,
};

const CONNECT: Option<&str> = option_env!("CONNECT");

fn callback_1(sample: &ZSample) {
    zenoh_nostd::info!(
        "[Subscription] Received Sample ('{}': '{:?}')",
        sample.keyexpr().as_str(),
        core::str::from_utf8(sample.payload()).unwrap()
    );
}

// #[embassy_executor::task]
// async fn callback_2() {
//     loop {
//         let sample = SUBSCRIBER_CHANNEL.receive().await;
//         zenoh_nostd::info!(
//             "[Subscription] Received Sample ('demo/example': '{:?}')",
//             core::str::from_utf8(sample).unwrap()
//         );
//     }
// }

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_sub example");

    let mut session = zenoh_nostd::open!(
        PlatformStd: (spawner, PlatformStd {}),
        TX: 512,
        RX: 512,
        CALLBACKS: 2,
        EndPoint::try_from(CONNECT.unwrap_or("tcp/127.0.0.1:7447")).unwrap()
    )
    .unwrap();

    let ke: &'static keyexpr = "demo/example/**".try_into().unwrap();

    let _subscriber = session
        .declare_subscriber(ke, zcallback!(callback_1))
        .await
        .unwrap();

    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
