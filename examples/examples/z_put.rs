#![cfg_attr(
    not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_arch = "wasm32",
    )),
    no_std
)]
#![cfg_attr(target_arch = "xtensa", no_main)]

#[cfg(target_arch = "xtensa")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

use core::str::FromStr;

use zenoh::{keyexpr, EndPoint};

#[cfg_attr(target_arch = "xtensa", esp_hal_embassy::main)]
#[cfg_attr(not(target_arch = "xtensa"), embassy_executor::main)]
async fn main(spawner: embassy_executor::Spawner) {
    zenoh::init_logger();

    zenoh::log::info!("Start z_put example");

    let mut session = zenoh::api::session::SingleLinkClientSession::open(
        EndPoint::from_str("tcp/127.0.0.1:7447").unwrap(),
        spawner,
    )
    .await
    .unwrap();

    let ke: &'static keyexpr = "demo/example".try_into().unwrap();
    let payload = b"Hello, world!";

    loop {
        session.try_read().unwrap();

        session.put(ke, payload).await.unwrap();
        zenoh::log::info!(
            "[Publisher] Sent PUT ('{}': '{}')",
            ke.as_str(),
            core::str::from_utf8(payload).unwrap()
        );

        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
