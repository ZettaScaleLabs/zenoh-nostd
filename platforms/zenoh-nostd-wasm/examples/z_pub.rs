#![no_main]

use {
    embassy_executor::Spawner,
    zenoh_nostd::{EndPoint, ZResult, keyexpr},
    zenoh_nostd_wasm::PlatformWasm,
};

const CONNECT: Option<&str> = option_env!("CONNECT");

async fn entry(spawner: Spawner) -> ZResult<()> {
    let mut i = 0usize;
    zenoh_nostd::info!("zenoh-nostd z_pub example");

    let config = zenoh_nostd::zconfig!(
            PlatformWasm: (spawner, PlatformWasm {}),
            TX: 512,
            RX: 512,
            MAX_SUBSCRIBERS: 2
    );

    let session = zenoh_nostd::open!(
        config,
        EndPoint::try_from(CONNECT.unwrap_or("ws/127.0.0.1:7446"))?
    );

    let publisher = session.declare_publisher(keyexpr::new("demo/example")?);

    loop {
        let payload = format!("[{}] Hello, publishing from no-std!", i);
        publisher.put(payload.as_bytes()).await?;

        zenoh_nostd::info!(
            "[Publisher] Sent PUT ('{}': '{}')",
            publisher.keyexpr().as_str(),
            payload
        );

        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
        i += 1;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh_nostd::error!("Error in main: {:?}", e);
    }

    zenoh_nostd::info!("Exiting main");
}
