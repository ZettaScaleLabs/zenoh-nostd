#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use zenoh_examples::*;
use zenoh_nostd as zenoh;

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

async fn callback(query: &zenoh::Query<'_, '_, ExampleConfig>) {
    match query.payload() {
        None => {
            zenoh::info!(
                "[Queryable] Received Query ('{}' with no payload)",
                query.keyexpr().as_str()
            );
        }
        Some(payload) => {
            zenoh::info!(
                "[Queryable] Received Query ('{}': '{:?}')",
                query.keyexpr().as_str(),
                core::str::from_utf8(payload).unwrap()
            );
        }
    }

    zenoh::info!("[Queryable] Sending OK Reply");
    let _ = query
        .reply(query.keyexpr(), b"Response from z_queryable")
        .await;

    let _ = query.finalize().await;
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh::info!("zenoh-nostd z_queryable example");

    let config = init_example(&spawner).await;
    let mut resources = zenoh::Resources::new();
    let session = zenoh::open(&mut resources, config, zenoh::EndPoint::try_from(CONNECT)?).await?;

    // session
    //     .declare_queryable(zenoh::keyexpr::new("demo/example/**")?)
    //     .callback(Callback::new_async(callback))
    //     .finish()
    //     .await?;

    session.run().await
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
