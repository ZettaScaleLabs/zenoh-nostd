#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use zenoh_examples::*;
use zenoh_nostd as zenoh;

async fn entry(spawner: embassy_executor::Spawner) -> zenoh::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh::info!("zenoh-nostd z_sub example");

    // All channels that will be used must outlive `Resources`.
    // **Note**: as a direct implication, you may need to make static channels
    // if you want a `'static` session.
    let channel = embassy_sync::channel::Channel::<
        embassy_sync::blocking_mutex::raw::NoopRawMutex,
        zenoh::OwnedSample<128, 128>,
        8,
    >::new();

    let config = init_example(&spawner).await;
    let mut resources = zenoh::Resources::default();

    let session = zenoh::open(&mut resources, config, zenoh::EndPoint::try_from(CONNECT)?).await?;

    let subscriber = session
        .declare_subscriber(zenoh::keyexpr::new("demo/example/**")?)
        .channel(channel.dyn_sender(), channel.dyn_receiver())
        .finish()
        .await?;

    embassy_futures::select::select(session.run(), async {
        while let Some(sample) = subscriber.recv().await {
            zenoh::info!(
                "[Subscriber] Received sample ('{}': '{}')",
                sample.keyexpr().as_str(),
                core::str::from_utf8(sample.payload()).unwrap()
            );
        }

        Ok::<(), zenoh::Error>(())
    })
    .await;

    Ok(())
}

#[cfg_attr(feature = "std", embassy_executor::main)]
#[cfg_attr(feature = "wasm", embassy_executor::main)]
#[cfg_attr(feature = "esp32s3", esp_rtos::main)]
async fn main(spawner: embassy_executor::Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh::error!("Error in main: {}", e);
    }

    zenoh::info!("Exiting main");
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
