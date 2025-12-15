#![cfg_attr(feature = "esp32s3", no_std)]
#![cfg_attr(feature = "esp32s3", no_main)]
#![cfg_attr(feature = "wasm", no_main)]

use embassy_time::Instant;
use static_cell::StaticCell;
use zenoh_examples::*;
use zenoh_nostd::api::*;

struct Stats {
    round_count: usize,
    round_size: usize,
    finished_rounds: usize,
    round_start: Instant,
    global_start: Option<Instant>,
}

impl Stats {
    fn increment(&mut self) {
        if self.round_count == 0 {
            self.round_start = Instant::now();
            if self.global_start.is_none() {
                self.global_start = Some(self.round_start)
            }
            self.round_count += 1;
        } else if self.round_count < self.round_size {
            self.round_count += 1;
        } else {
            self.print_round();
            self.finished_rounds += 1;
            self.round_count = 0;
        }
    }

    fn print_round(&self) {
        let elapsed = self.round_start.elapsed().as_micros() as f64 / 1_000_000.0;
        let throughput = (self.round_size as f64) / elapsed;
        zenoh_nostd::info!("{} msg/s", throughput);
    }
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_sub_thr example");

    let config = init_example(&spawner).await;
    static RESOURCES: StaticCell<Resources<ExampleConfig>> = StaticCell::new();
    let session = zenoh_nostd::api::open(
        RESOURCES.init(Resources::new()),
        config,
        EndPoint::try_from(CONNECT)?,
    )
    .await?;

    let mut stats = Stats {
        round_count: 0,
        round_size: 100_000,
        finished_rounds: 0,
        round_start: Instant::now(),
        global_start: None,
    };

    let _ = session
        .declare_subscriber(keyexpr::new("test/thr")?)
        .callback(Callback::new_sync(move |_| {
            if stats.finished_rounds >= 10000 {
                // TODO! implement a `session.close()` that can be called here to gracefully terminate the session
                return;
            }

            stats.increment();
        }))
        .finish()
        .await?;

    session.run().await
}

#[cfg_attr(feature = "std", embassy_executor::main)]
#[cfg_attr(feature = "esp32s3", esp_rtos::main)]
#[cfg_attr(feature = "wasm", embassy_executor::main)]
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
