#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use static_cell::StaticCell;
use zenoh_examples::*;
use zenoh_nostd::api::*;

#[embassy_executor::task]
async fn session_task(session: Session<'static, ExampleConfig>) {
    if let Err(e) = session.run().await {
        zenoh_nostd::error!("Error in session task: {}", e);
    }
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_pong example");

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
        .declare_subscriber(keyexpr::new("test/ping")?)
        .finish()
        .await?;

    let pong = session
        .declare_publisher(keyexpr::new("test/pong")?)
        .finish()
        .await?;

    while let Ok(sample) = ping.recv().await {
        pong.put(sample.payload()).finish().await?;
    }

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
