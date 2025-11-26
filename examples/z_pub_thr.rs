#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use embassy_time::Instant;
use zenoh_examples::*;
use zenoh_nostd::{EndPoint, keyexpr};

const CONNECT: &str = match option_env!("CONNECT") {
    Some(v) => v,
    None => {
        if cfg!(feature = "wasm") {
            "ws/127.0.0.1:7446"
        } else {
            "tcp/127.0.0.1:7447"
        }
    }
};

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

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_pub_thr example");

    let platform = init_platform(&spawner).await;
    let config = zenoh_nostd::zconfig!(
            Platform: (spawner, platform),
            TX: 512,
            RX: 512,
            MAX_SUBSCRIBERS: 2,
            MAX_QUERIES: 2,
            MAX_QUERYABLES: 2
    );

    let session = zenoh_nostd::open!(config, EndPoint::try_from(CONNECT)?);

    let mut payload = [0u8; PAYLOAD];
    for (i, p) in payload.iter_mut().enumerate().take(PAYLOAD) {
        *p = (i % 10) as u8;
    }

    let publisher = session.declare_publisher(keyexpr::new("test/thr")?);

    let mut count: usize = 0;
    let mut start = Instant::now();
    loop {
        publisher.put(&payload).await?;

        if count < 100_000 {
            count += 1;
        } else {
            let thpt = count as f64 / (start.elapsed().as_micros() as f64 / 1_000_000.0);
            zenoh_nostd::info!("{} msgs/s", thpt);
            count = 0;
            start = Instant::now();
        }
    }
}

#[cfg_attr(feature = "std", embassy_executor::main)]
#[cfg_attr(feature = "esp32s3", esp_rtos::main)]
#[cfg_attr(feature = "wasm", embassy_executor::main)]
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
