#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use zenoh_examples::*;
use zenoh_nostd::api::*;

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_sub example");

    let config = init_example(&spawner).await;
    let mut resources = Resources::new();
    let session =
        zenoh_nostd::api::open(&mut resources, config, EndPoint::try_from(CONNECT)?).await?;

    // In this example we care about maintaining the session alive, we then have two choices:
    //  1) Spawn a new task to run the `session.run()` in background, but it requires the `resources` to be `static`.
    //  2) Use `select` or `join` to run both the session and the subscriber in the same task.
    // Here we use the second approach. For a demonstration of the first approach, see the `z_open` example.

    let subscriber = session
        .declare_subscriber(keyexpr::new("demo/example/**")?)
        .finish()
        .await?;

    embassy_futures::select::select(session.run(), async {
        while let Ok(sample) = subscriber.recv().await {
            zenoh_nostd::info!(
                "[Subscriber] Received sample ('{}': '{}')",
                sample.keyexpr().as_str(),
                core::str::from_utf8(sample.payload()).unwrap()
            );
        }

        Ok::<(), zenoh_nostd::Error>(())
    })
    .await;

    zenoh_nostd::info!("[Subscriber] Undeclaring subscriber and exiting...");
    subscriber.undeclare().await?;

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
