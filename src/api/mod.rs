use embassy_executor::{SpawnToken, Spawner};
use static_cell::StaticCell;

use crate::{
    api::{driver::SessionDriver, subscriber::ZSubscriberCallbacks},
    io::transport::Transport,
    platform::Platform,
};

pub mod callback;
pub mod sample;

pub mod driver;
pub mod publisher;
pub mod session;
pub mod subscriber;

pub struct ZConfig<T: Platform + 'static, S> {
    pub spawner: Spawner,
    pub platform: T,
    pub task: fn(driver: &'static SessionDriver<T>) -> SpawnToken<S>,

    pub driver: &'static StaticCell<SessionDriver<T>>,
    pub transport: &'static StaticCell<Transport<T>>,

    pub tx_zbuf: &'static mut [u8],
    pub rx_zbuf: &'static mut [u8],

    pub subscribers: &'static mut dyn ZSubscriberCallbacks,
}

#[macro_export]
macro_rules! zconfig {
    ($type:ident : ($spawner:expr, $platform:expr), TX: $TX:expr, RX: $RX:expr, SUBSCRIBERS: $SUBSCRIBERS:expr) => {{
        static DRIVER: static_cell::StaticCell<$crate::api::driver::SessionDriver<$type>> =
            static_cell::StaticCell::new();

        static TRANSPORT: static_cell::StaticCell<$crate::io::transport::Transport<$type>> =
            static_cell::StaticCell::new();

        static TX_ZBUF: static_cell::StaticCell<[u8; $TX]> = static_cell::StaticCell::new();
        static RX_ZBUF: static_cell::StaticCell<[u8; $RX]> = static_cell::StaticCell::new();

        static SUBSCRIBERS: static_cell::StaticCell<
            $crate::api::subscriber::ZSubscriberCallbackStorage<$SUBSCRIBERS>,
        > = static_cell::StaticCell::new();

        #[embassy_executor::task]
        async fn session_task(runner: &'static $crate::api::driver::SessionDriver<$type>) {
            if let Err(e) = runner.run().await {
                $crate::error!("Session driver task ended with error: {}", e);
            }
        }

        let zconfig = $crate::api::ZConfig {
            spawner: $spawner,
            platform: $platform,
            task: session_task,

            driver: &DRIVER,
            transport: &TRANSPORT,

            tx_zbuf: TX_ZBUF.init([0u8; $TX]).as_mut_slice(),
            rx_zbuf: RX_ZBUF.init([0u8; $RX]).as_mut_slice(),
            subscribers: SUBSCRIBERS.init($crate::api::subscriber::ZSubscriberCallbackStorage::<
                $SUBSCRIBERS,
            >::new()),
        };

        zconfig
    }};
}
