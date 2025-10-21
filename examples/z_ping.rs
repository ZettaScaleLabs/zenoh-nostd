use embassy_time::{Duration, Instant};
use zenoh_nostd::{EndPoint, PlatformStd, keyexpr, zsubscriber};

const CONNECT: Option<&str> = option_env!("CONNECT");

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    #[cfg(feature = "log")]
    env_logger::init();

    zenoh_nostd::info!("zenoh-nostd z_ping example");

    let mut session = zenoh_nostd::open!(
        zenoh_nostd::zconfig!(
                PlatformStd: (spawner, PlatformStd {}),
                TX: 512,
                RX: 512,
                MAX_SUBSCRIBERS: 2
        ),
        EndPoint::try_from(CONNECT.unwrap_or("tcp/127.0.0.1:7447")).unwrap()
    )
    .unwrap();

    let ke_pong: &'static keyexpr = "test/pong".try_into().unwrap();
    let ke_ping: &'static keyexpr = "test/ping".try_into().unwrap();

    let sub = session
        .declare_subscriber(
            ke_pong,
            zsubscriber!(QUEUE_SIZE: 8, MAX_KEYEXPR: 32, MAX_PAYLOAD: 128),
        )
        .await
        .unwrap();

    let data = [0, 1, 2, 3, 4, 5, 6, 7];

    let mut samples = Vec::<u64>::with_capacity(100);

    zenoh_nostd::info!("Warming up for 1s");
    let now = Instant::now();

    while now.elapsed() < Duration::from_secs(1) {
        session.put(ke_ping, &data).await.unwrap();

        let _ = sub.recv().await.unwrap();
    }

    zenoh_nostd::info!("Starting ping-pong measurements");

    for _ in 0..100 {
        let start = Instant::now();

        session.put(ke_ping, &data).await.unwrap();

        let _ = sub.recv().await.unwrap();

        let elapsed = start.elapsed().as_micros();
        samples.push(elapsed);
    }

    for (i, rtt) in samples.iter().enumerate().take(100) {
        println!("{} bytes: seq={} rtt={:?}µs lat={:?}µs", 8, i, rtt, rtt / 2);
    }

    let avg_rtt: u64 = samples.iter().sum::<u64>() as u64 / samples.len() as u64;
    let avg_lat: u64 = avg_rtt / 2;

    println!(
        "Average RTT: {:?}µs, Average Latency: {:?}µs",
        avg_rtt, avg_lat
    );
}
