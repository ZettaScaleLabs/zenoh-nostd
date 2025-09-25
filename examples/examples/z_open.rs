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

use zenoh::EndPoint;

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    zenoh::init_logger();

    zenoh::log::info!("Start z_open example");

    let mut session = zenoh::api::session::SingleLinkClientSession::open(
        EndPoint::from_str("tcp/127.0.0.1:7447").unwrap(),
        spawner,
    )
    .await
    .unwrap();

    loop {
        session.read().await.unwrap();
    }
}
