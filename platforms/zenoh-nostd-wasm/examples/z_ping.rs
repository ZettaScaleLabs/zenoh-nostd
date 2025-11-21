#![no_main]

use {
    embassy_executor::Spawner,
    embassy_time::{Duration, Instant},
    zenoh_nostd::{EndPoint, keyexpr, zsubscriber},
    zenoh_nostd_wasm::PlatformWasm,
};

const CONNECT: Option<&str> = option_env!("CONNECT");

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    zenoh_nostd::info!("zenoh-nostd z_ping example");
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
            ke_pong,
            zsubscriber!(QUEUE_SIZE: 8, MAX_KEYEXPR: 32, MAX_PAYLOAD: 128),
        )
        .await?;

    let data = [0, 1, 2, 3, 4, 5, 6, 7];

    let mut samples = Vec::<u64>::with_capacity(100);

    zenoh_nostd::info!("Warming up for 1s");
    let now = Instant::now();

    while now.elapsed() < Duration::from_secs(1) {
        session.put(ke_ping, &data).await?;

        let _ = sub.recv().await?;
    }

    zenoh_nostd::info!("Starting ping-pong measurements");

    for _ in 0..100 {
        let start = Instant::now();

        session.put(ke_ping, &data).await?;

        let _ = sub.recv().await?;

        let elapsed = start.elapsed().as_micros();
        samples.push(elapsed);
    }

    for (i, rtt) in samples.iter().enumerate().take(100) {
        zenoh_nostd::info!("{} bytes: seq={} rtt={:?}µs lat={:?}µs", 8, i, rtt, rtt / 2);
    }

    let avg_rtt: u64 = samples.iter().sum::<u64>() / samples.len() as u64;
    let avg_lat: u64 = avg_rtt / 2;

    zenoh_nostd::info!(
        "Average RTT: {:?}µs, Average Latency: {:?}µs",
        avg_rtt,
        avg_lat
    );

    Ok(())
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh_nostd::error!("Error in main: {:?}", e);
    }

    zenoh_nostd::info!("Exiting main");
}
