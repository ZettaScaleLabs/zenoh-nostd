#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use zenoh_examples::*;
use zenoh_nostd::{api::*, *};

zimport_types!(
    PLATFORM: Platform,
    TX: [u8; 512],
    RX: [u8; 512],

    MAX_KEYEXPR_LEN: 64,
    MAX_PARAMETERS_LEN: 128,
    MAX_PAYLOAD_LEN: 512,

    MAX_QUEUED: 8,
    MAX_CALLBACKS: 8,

    MAX_SUBSCRIBERS: 8,
);

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

fn callback_1(sample: &Sample) {
    zenoh_nostd::info!(
        "[Subscriber] Received Sample ('{}': '{:?}')",
        sample.keyexpr().as_str(),
        core::str::from_utf8(sample.payload()).unwrap()
    );
}

#[embassy_executor::task]
async fn callback_2(mut subscriber: Subscriber<'static>) {
    while let Some(sample) = subscriber.recv().await {
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
    let config = Config::new(platform, [0u8; 512], [0u8; 512]);

    let mut resources = Resources::new();
    let cb1 = resources.subscriber_sync(callback_1).await?;
    let cb2 = resources.subscriber_async().await?;

    let session =
        zenoh_nostd::api::open(&mut resources, config, EndPoint::try_from(CONNECT)?).await?;

    // In this example we care about maintaining the session alive, we then have two choices:
    //  1) Spawn a new task to run the `session.run()` in background, but it requires the `resources` to be `static`.
    //  2) Use `select` or `join` to run both the session and the subscriber in the same task.
    // Here we use the second approach. For a demonstration of the first approach, see the `z_queryable` example.

    let ke = keyexpr::new("demo/example/**")?;

    let _ = session.declare_subscriber(ke, cb1).finish().await?;
    let mut subscriber = session.declare_subscriber(ke, cb2).finish().await?;

    embassy_futures::select::select(session.run(), async {
        loop {
            while let Some(sample) = subscriber.recv().await {
                zenoh_nostd::info!(
                    "[Async Subscriber] Received Sample ('{}': '{:?}')",
                    sample.keyexpr().as_str(),
                    core::str::from_utf8(sample.payload()).unwrap()
                );
            }
        }
    })
    .await;

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
