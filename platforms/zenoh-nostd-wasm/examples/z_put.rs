use embassy_executor::Spawner;
use zenoh_nostd::{keyexpr::borrowed::keyexpr, protocol::core::endpoint::EndPoint};
use zenoh_nostd_wasm::PlatformWasm;

const CONNECT: Option<&str> = option_env!("CONNECT");

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    zenoh_nostd::info!("zenoh-nostd z_put example");

    let mut session = zenoh_nostd::open!(
        PlatformWasm: (spawner, PlatformWasm {}),
        EndPoint::try_from(CONNECT.unwrap_or("ws/127.0.0.1:7447")).unwrap()
    )
    .unwrap();

    let ke: &'static keyexpr = "demo/example".try_into().unwrap();
    let payload = b"Hello, from wasm!";

    let mut tx_zbuf = [0u8; 64];

    loop {
        session
            .put(tx_zbuf.as_mut_slice(), ke, payload)
            .await
            .unwrap();

        zenoh_nostd::info!(
            "[Publisher] Sent PUT ('{}': '{}')",
            ke.as_str(),
            core::str::from_utf8(payload).unwrap()
        );

        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
