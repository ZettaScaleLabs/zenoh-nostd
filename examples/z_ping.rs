#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]

use embassy_time::{Duration, Instant};
use zenoh_examples::*;
use zenoh_nostd::{EndPoint, keyexpr, zsubscriber};

const CONNECT: Option<&str> = option_env!("CONNECT");

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_ping example");

    let platform = init_platform(&spawner).await;
    let config = zenoh_nostd::zconfig!(
            Platform: (spawner, platform),
            TX: 512,
            RX: 512,
            MAX_SUBSCRIBERS: 2,
            MAX_QUERIES: 2
    );

    let mut session = zenoh_nostd::open!(
        config,
        EndPoint::try_from(CONNECT.unwrap_or("tcp/127.0.0.1:7447"))?
    );

    let ke_pong = keyexpr::new("test/pong").unwrap();
    let ke_ping = keyexpr::new("test/ping").unwrap();

    let sub = session
        .declare_subscriber(
            ke_pong,
            zsubscriber!(QUEUE_SIZE: 8, MAX_KEYEXPR: 32, MAX_PAYLOAD: 128),
        )
        .await
        .unwrap();

    let data = [0, 1, 2, 3, 4, 5, 6, 7];

    #[cfg(feature = "esp32s3")]
    extern crate alloc;
    #[cfg(feature = "esp32s3")]
    use alloc::vec::Vec;

    let mut samples = Vec::<u64>::with_capacity(100);

    zenoh_nostd::info!("Warming up for 1s");
    let now = Instant::now();

    while now.elapsed() < Duration::from_secs(1) {
        session.put(ke_ping, &data).await.unwrap();

        let _ = sub.recv().await.unwrap();
    }

    zenoh_nostd::info!("Starting ping-pong measurements");

    for _ in 0..100 {
        let start = Instant::now();

        session.put(ke_ping, &data).await.unwrap();

        let _ = sub.recv().await.unwrap();

        let elapsed = start.elapsed().as_micros();
        samples.push(elapsed);
    }

    for (i, rtt) in samples.iter().enumerate().take(100) {
        zenoh_nostd::info!("{} bytes: seq={} rtt={:?}µs lat={:?}µs", 8, i, rtt, rtt / 2);
    }

    let avg_rtt: u64 = samples.iter().sum::<u64>() / samples.len() as u64;
    let avg_lat: u64 = avg_rtt / 2;

    zenoh_nostd::info!(
        "Average RTT: {:?}µs, Average Latency: {:?}µs",
        avg_rtt,
        avg_lat
    );

    Ok(())
}

#[cfg_attr(feature = "std", embassy_executor::main)]
#[cfg_attr(feature = "esp32s3", esp_rtos::main)]
async fn main(spawner: embassy_executor::Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh_nostd::error!("Error in main: {:?}", e);
    }

    zenoh_nostd::info!("Exiting main");
}

#[cfg(feature = "esp32s3")]
mod esp32s3_app {
    use esp_hal::rng::Rng;
    pub use esp_println as _;
    use getrandom::{Error, register_custom_getrandom};

    #[panic_handler]
    fn panic(info: &core::panic::PanicInfo) -> ! {
        zenoh_nostd::error!("Panic: {}", info);

        loop {}
    }

    extern crate alloc;

    esp_bootloader_esp_idf::esp_app_desc!();

    register_custom_getrandom!(getrandom_custom);
    pub fn getrandom_custom(bytes: &mut [u8]) -> Result<(), Error> {
        Rng::new().read(bytes);
        Ok(())
    }
}
