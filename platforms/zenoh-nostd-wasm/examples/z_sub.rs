use embassy_executor::Spawner;
use zenoh_nostd::{keyexpr::borrowed::keyexpr, protocol::core::endpoint::EndPoint};
use zenoh_nostd_wasm::PlatformWasm;

const CONNECT: Option<&str> = option_env!("CONNECT");

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    zenoh_nostd::info!("zenoh-nostd z_sub example");

    let mut session = zenoh_nostd::open!(
        PlatformWasm: (spawner, PlatformWasm {}),
        EndPoint::try_from(CONNECT.unwrap_or("ws/127.0.0.1:7447")).unwrap()
    )
    .unwrap();

    let ke: &'static keyexpr = "demo/example".try_into().unwrap();

    let mut tx_zbuf = [0u8; 256];
    let subscription_1 = session
        .declare_subscription(tx_zbuf.as_mut_slice(), ke)
        .await
        .unwrap();

    let mut rx_buffer = [0u8; 512];
    loop {
        session
            .read(rx_buffer.as_mut_slice(), async |subscription, sample| {
                if subscription == subscription_1 {
                    zenoh_nostd::info!(
                        "[Subscription] Received Sample ('{}': '{:?}')",
                        ke.as_str(),
                        core::str::from_utf8(sample).unwrap()
                    );
                }
            })
            .await
            .unwrap();
    }
}
