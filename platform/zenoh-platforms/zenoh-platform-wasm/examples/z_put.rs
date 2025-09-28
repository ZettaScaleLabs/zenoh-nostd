use core::str::FromStr;

use static_cell::StaticCell;
use zenoh::{api::session::SessionRunner, keyexpr, EndPoint};
use zenoh_platform_wasm::PlatformWasm;

#[embassy_executor::task]
async fn session_task(mut runner: SessionRunner<'static, PlatformWasm>) {
    runner.run().await;
}

#[embassy_executor::main]
pub async fn main(spawner: embassy_executor::Spawner) {
    web_sys::console::log_1(&"Starting z_put example...".into());

    static PLATFORM: StaticCell<PlatformWasm> = StaticCell::new();
    let platform = PLATFORM.init(PlatformWasm);

    let (mut session, runner) = zenoh::api::session::SingleLinkClientSession::open(
        platform,
        EndPoint::from_str("ws/127.0.0.1:7447").unwrap(),
    )
    .await
    .unwrap();

    spawner.spawn(session_task(runner)).unwrap();

    let ke: &'static keyexpr = "demo/example".try_into().unwrap();
    let payload = b"Hello, world!";

    loop {
        session.try_read().unwrap();

        session.put(ke, payload).await.unwrap();

        web_sys::console::log_1(
            &format!(
                "[Publisher] Sent PUT ('{}': '{}')",
                ke.as_str(),
                core::str::from_utf8(payload).unwrap()
            )
            .into(),
        );

        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
