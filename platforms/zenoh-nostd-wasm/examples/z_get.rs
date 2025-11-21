#![no_main]

use {
    embassy_executor::Spawner,
    zenoh_nostd::{EndPoint, ZReply, ZResult, keyexpr},
    zenoh_nostd_wasm::PlatformWasm,
};

const CONNECT: Option<&str> = option_env!("CONNECT");

fn callback(reply: &ZReply) {
    match reply {
        ZReply::Ok(reply) => {
            zenoh_nostd::info!(
                "[Query] Received OK Reply ('{}': '{:?}')",
                reply.keyexpr().as_str(),
                core::str::from_utf8(reply.payload()).unwrap()
            );
        }
        ZReply::Err(reply) => {
            zenoh_nostd::error!(
                "[Query] Received ERR Reply ('{}': '{:?}')",
                reply.keyexpr().as_str(),
                core::str::from_utf8(reply.payload()).unwrap()
            );
        }
    }
}

async fn entry(spawner: embassy_executor::Spawner) -> zenoh_nostd::ZResult<()> {
    zenoh_nostd::info!("zenoh-nostd z_get example");
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

    let ke = keyexpr::new("demo/example/**").unwrap();

    // Because of memory growth concerns with async channels, `session.get`
    // only supports callback-based usage in `zenoh-nostd`.
    session.get(ke, callback).send().await.unwrap();

    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh_nostd::error!("Error in main: {:?}", e);
    }

    zenoh_nostd::info!("Exiting main");
}
