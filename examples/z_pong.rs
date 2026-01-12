#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use zenoh_examples::*;
use zenoh_nostd as zenoh;

#[embassy_executor::task]
async fn session_task(session: &'static zenoh::Session<'static, ExampleConfig>) {
    if let Err(e) = session.run().await {
        zenoh::error!("Error in session task: {}", e);
    }
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh::info!("zenoh-nostd z_pong example");

    // All channels that will be used must outlive `Resources`.
    // **Note**: as a direct implication, here you need to make a static channel.
    static CHANNEL: static_cell::StaticCell<
        embassy_sync::channel::Channel<
            embassy_sync::blocking_mutex::raw::NoopRawMutex,
            zenoh::OwnedSample<128, 128>,
            8,
        >,
    > = static_cell::StaticCell::new();
    let channel = CHANNEL.init(embassy_sync::channel::Channel::new());

    let config = init_example(&spawner).await;
    let session = zenoh::open!(config => ExampleConfig, zenoh::EndPoint::try_from(CONNECT)?);

    spawner.spawn(session_task(session)).map_err(|e| {
        zenoh::error!("Error spawning task: {}", e);
        zenoh::SessionError::CouldNotSpawnEmbassyTask
    })?;

    let ping = session
        .declare_subscriber(zenoh::keyexpr::new("test/ping")?)
        .channel(channel.dyn_sender(), channel.dyn_receiver())
        .finish()
        .await?;

    let pong = session
        .declare_publisher(zenoh::keyexpr::new("test/pong")?)
        .finish()
        .await?;

    while let Some(sample) = ping.recv().await {
        pong.put(sample.payload()).finish().await?;
    }

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
        zenoh::error!("Panic: {}", info);

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
