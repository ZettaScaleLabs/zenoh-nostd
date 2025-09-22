use core::str::FromStr;

use xcutor::*;

use zenoh_nano::*;

fn main() {
    block_on(async move {
        let session = zenoh_nano::session::open(EndPoint::from_str("tcp/127.0.0.1:7447").unwrap())
            .await
            .unwrap();

        session.lease_task().await.unwrap();
    })
}
