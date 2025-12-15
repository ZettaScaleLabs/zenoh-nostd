#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use embassy_time::{Duration, Instant};
use static_cell::StaticCell;
use zenoh_examples::*;
use zenoh_nostd::api::*;

const PAYLOAD: usize = match usize::from_str_radix(
    match option_env!("PAYLOAD") {
        Some(v) => v,
        None => "8",
    },
    10,
) {
    Ok(v) => v,
    Err(_) => 8,
};

#[embassy_executor::task]
async fn session_task(session: Session<'static, ExampleConfig>) {
    if let Err(e) = session.run().await {
        zenoh_nostd::error!("Error in session task: {}", e);
    }
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_ping example");

    let config = init_example(&spawner).await;
    static RESOURCES: StaticCell<Resources<ExampleConfig>> = StaticCell::new();
    let session = zenoh_nostd::api::open(
        RESOURCES.init(Resources::new()),
        config,
        EndPoint::try_from(CONNECT)?,
    )
    .await?;

    spawner.spawn(session_task(session.clone())).map_err(|e| {
        zenoh_nostd::error!("Error spawning task: {}", e);
        zenoh_nostd::SessionError::CouldNotSpawnEmbassyTask
    })?;

    let ping = session
        .declare_publisher(keyexpr::new("test/ping")?)
        .finish()
        .await?;

    let pong = session
        .declare_subscriber(keyexpr::new("test/pong")?)
        .finish()
        .await?;

    let data: [u8; PAYLOAD] = core::array::from_fn(|i| (i % 10) as u8);
    let mut samples = [0u64; 100];

    zenoh_nostd::info!("Warming up for 1s");
    let now = Instant::now();

    while now.elapsed() < Duration::from_secs(1) {
        ping.put(&data).finish().await?;
        let _ = pong.recv().await?;
    }

    zenoh_nostd::info!("Starting ping-pong measurements");

    for sample in samples.iter_mut() {
        let start = Instant::now();

        ping.put(&data).finish().await?;
        let _ = pong.recv().await?;

        *sample = start.elapsed().as_micros();
    }

    for (i, rtt) in samples.iter().enumerate() {
        zenoh_nostd::info!(
            "{} bytes: seq={} rtt={:?}µs lat={:?}µs",
            data.len(),
            i,
            rtt,
            rtt / 2
        );
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
#[cfg_attr(feature = "wasm", embassy_executor::main)]
#[cfg_attr(feature = "esp32s3", esp_rtos::main)]
async fn main(spawner: embassy_executor::Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh_nostd::error!("Error in main: {}", e);
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
