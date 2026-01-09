#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use embassy_futures::join::join;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use zenoh_examples::*;
use zenoh_nostd::{self as zenoh, OwnedResponse, Response};

async fn response_callback(resp: &Response<'_>) {
    match resp {
        Response::Ok(reply) => {
            zenoh_nostd::info!(
                "[Get] Received OK Reply ('{}': '{:?}')",
                reply.keyexpr().as_str(),
                core::str::from_utf8(reply.payload()).unwrap()
            );
        }
        Response::Err(reply) => {
            zenoh_nostd::error!(
                "[Get] Received ERR Reply ('{}': '{:?}')",
                reply.keyexpr().as_str(),
                core::str::from_utf8(reply.payload()).unwrap()
            );
        }
    }
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh::info!("zenoh-nostd z_get example");

    let config = init_example(&spawner).await;

    // All resources that will be used must outlive `Resources`.
    let channel = Channel::<NoopRawMutex, OwnedResponse<128, 128>, 8>::new();

    let mut resources = zenoh::Resources::new();
    let session = zenoh::open(&mut resources, config, zenoh::EndPoint::try_from(CONNECT)?).await?;

    let responses = session
        .get(zenoh::keyexpr::new("demo/example/**")?)
        // .callback(response_callback)
        .channel(channel.dyn_sender(), channel.dyn_receiver())
        .finish()
        .await?;

    join(session.run(), async {
        while let Some(response) = responses.recv().await {
            response_callback(&response.as_ref()).await
        }
    })
    .await
    .0
    .ok();

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
