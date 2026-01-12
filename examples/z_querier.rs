#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use zenoh_examples::*;
use zenoh_nostd::{self as zenoh};

fn response_callback(resp: &zenoh::Response<'_>) {
    match resp {
        zenoh::Response::Ok(reply) => {
            zenoh_nostd::info!(
                "[Get] Received OK Reply ('{}': '{:?}')",
                reply.keyexpr().as_str(),
                core::str::from_utf8(reply.payload()).unwrap()
            );
        }
        zenoh::Response::Err(reply) => {
            zenoh_nostd::error!(
                "[Get] Received ERR Reply ('{}': '{:?}')",
                reply.keyexpr().as_str(),
                core::str::from_utf8(reply.payload()).unwrap()
            );
        }
    }
}

#[embassy_executor::task]
async fn session_task(session: zenoh::Session<'static, 'static, ExampleConfig>) {
    if let Err(e) = session.run().await {
        zenoh::error!("Error in session task: {}", e);
    }
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh::info!("zenoh-nostd z_get example");

    let config = init_example(&spawner).await;
    let session =
        zenoh::open!(config => ExampleConfig, zenoh::EndPoint::try_from(CONNECT)?).await?;

    spawner.spawn(session_task(session.clone())).map_err(|e| {
        zenoh::error!("Error spawning task: {}", e);
        zenoh::SessionError::CouldNotSpawnEmbassyTask
    })?;

    let querier = session
        .declare_querier(zenoh::keyexpr::new("demo/example/**")?)
        .timeout(embassy_time::Duration::from_secs(1))
        .finish()
        .await?;

    loop {
        querier
            .get()
            .callback_sync(|resp| response_callback(resp))
            .finish()
            .await?;

        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
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
