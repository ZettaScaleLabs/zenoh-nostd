#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]

use zenoh_examples::*;
use zenoh_nostd::{EndPoint, ZReply, keyexpr, zreplies};

const CONNECT: Option<&str> = option_env!("CONNECT");

fn callback_1(reply: &ZReply) {
    match reply {
        ZReply::Ok(reply) => {
            zenoh_nostd::info!(
                "[Query] Received OK Reply ('{}': '{:?}')",
                reply.keyexpr().as_str(),
                core::str::from_utf8(reply.payload()).unwrap()
            );
        }
        ZReply::Err(reply) => {
            zenoh_nostd::error!(
                "[Query] Received ERR Reply ('{}': '{:?}')",
                reply.keyexpr().as_str(),
                core::str::from_utf8(reply.payload()).unwrap()
            );
        }
    }
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_put example");

    let platform = init_platform(&spawner).await;
    let config = zenoh_nostd::zconfig!(
            Platform: (spawner, platform),
            TX: 512,
            RX: 512,
            MAX_SUBSCRIBERS: 2,
            MAX_QUERIES: 2
    );

    let session = zenoh_nostd::open!(
        config,
        EndPoint::try_from(CONNECT.unwrap_or("tcp/127.0.0.1:7447"))?
    );

    let querier = session.declare_querier(keyexpr::new("demo/example/**")?);

    loop {
        let _ = querier
            .get(None, None, None, zreplies!(callback_1))
            .await
            .unwrap();

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
