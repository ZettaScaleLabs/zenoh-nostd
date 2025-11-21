#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]

use zenoh_examples::*;
use zenoh_nostd::{EndPoint, ZSample, ZSubscriber, keyexpr, zsubscriber};

const CONNECT: Option<&str> = option_env!("CONNECT");

fn callback_1(sample: &ZSample) {
    zenoh_nostd::info!(
        "[Subscriber] Received Sample ('{}': '{:?}')",
        sample.keyexpr().as_str(),
        core::str::from_utf8(sample.payload()).unwrap()
    );
}

#[embassy_executor::task]
async fn callback_2(subscriber: ZSubscriber<32, 128>) {
    while let Ok(sample) = subscriber.recv().await {
        zenoh_nostd::info!(
            "[Async Subscriber] Received Sample ('{}': '{:?}')",
            sample.keyexpr().as_str(),
            core::str::from_utf8(sample.payload()).unwrap()
        );
    }
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_sub example");

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

    let ke = keyexpr::new("demo/example/**").unwrap();

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
