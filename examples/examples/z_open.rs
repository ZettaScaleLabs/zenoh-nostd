use core::str::FromStr;

use embassy_executor::Spawner;
use zenoh_protocol::core::EndPoint;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut session = zenoh_nostd::api::session::SingleLinkClientSession::open(
        EndPoint::from_str("tcp/127.0.0.1:7447").unwrap(),
        spawner,
    )
    .await
    .unwrap();

    loop {
        session.read().await.unwrap();
    }
}
