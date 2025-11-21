#![no_main]

use {
    embassy_executor::Spawner,
    zenoh_nostd::{EndPoint, ZResult, keyexpr},
    zenoh_nostd_wasm::PlatformWasm,
};

const CONNECT: Option<&str> = option_env!("CONNECT");

async fn entry(spawner: embassy_executor::Spawner) -> ZResult<()> {
    zenoh_nostd::info!("zenoh-nostd z_put example");
    let config = zenoh_nostd::zconfig!(
            PlatformWasm: (spawner, PlatformWasm {}),
            TX: 512,
            RX: 512,
            MAX_SUBSCRIBERS: 2,
            MAX_QUERIES: 2
    );

    let mut session = zenoh_nostd::open!(
        config,
        EndPoint::try_from(CONNECT.unwrap_or("ws/127.0.0.1:7446"))?
    );

    let ke = keyexpr::new("demo/example").expect("Failed to create key expression");

    let payload = b"Hello, from wasm!";

    loop {
        session.put(ke, payload).await?;

        zenoh_nostd::info!(
            "[Publisher] Sent PUT ('{}': '{}')",
            ke.as_str(),
            core::str::from_utf8(payload).unwrap()
        );

        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh_nostd::error!("Error in main: {:?}", e);
    }

    zenoh_nostd::info!("Exiting main");
}
