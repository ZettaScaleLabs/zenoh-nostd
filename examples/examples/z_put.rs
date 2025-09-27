use core::str::FromStr;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use zenoh_protocol::core::{key_expr::keyexpr, EndPoint};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut session = zenoh_nostd::api::session::SingleLinkClientSession::open(
        EndPoint::from_str("tcp/127.0.0.1:7447").unwrap(),
        spawner,
    )
    .await
    .unwrap();

    let ke: &'static keyexpr = "demo/example".try_into().unwrap();

    loop {
        session.try_read().unwrap();

        session.put(ke, b"Hello, world!").await.unwrap();
        println!("Sent data");

        Timer::after(Duration::from_secs(1)).await;
    }
}
