#![no_main]

use {
    embassy_executor::Spawner,
    zenoh_nostd::{EndPoint, ZReply, ZResult, keyexpr, zsubscriber},
    zenoh_nostd_wasm::PlatformWasm,
};

const CONNECT: Option<&str> = option_env!("CONNECT");

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    zenoh_nostd::info!("zenoh-nostd z_pong example");
    let config = zenoh_nostd::zconfig!(
            PlatformWasm: (spawner, PlatformWasm {}),
            TX: 512,
            RX: 512,
            MAX_SUBSCRIBERS: 2,
            MAX_QUERIES: 2
    );

    let session = zenoh_nostd::open!(
        config,
        EndPoint::try_from(CONNECT.unwrap_or("ws/127.0.0.1:7446"))?
    );

    let ke_pong = keyexpr::new("test/pong")?;
    let ke_ping = keyexpr::new("test/ping")?;

    let sub = session
        .declare_subscriber(
            ke_ping,
            zsubscriber!(QUEUE_SIZE: 8, MAX_KEYEXPR: 32, MAX_PAYLOAD: 128),
        )
        .await?;

    while let Ok(sample) = sub.recv().await {
        session.put(ke_pong, sample.payload()).await?;
    }

    Ok(())
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh_nostd::error!("Error in main: {:?}", e);
    }

    zenoh_nostd::info!("Exiting main");
}
