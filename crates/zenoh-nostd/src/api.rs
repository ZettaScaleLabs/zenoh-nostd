use embassy_executor::{SpawnToken, Spawner};
use static_cell::StaticCell;

use crate::{
    io::transport::Transport, platform::Platform, replies::callback::ZRepliesCallbacks,
    subscriber::callback::ZSubscriberCallbacks,
};

pub(crate) mod sample;
pub use sample::{ZOwnedSample, ZSample};

pub(crate) mod reply;
pub use reply::{ZOwnedReply, ZReply};

pub(crate) mod driver;
pub use driver::SessionDriver;

pub(crate) mod session;
pub use session::Session;

pub(crate) mod subscriber;
pub use subscriber::{
    ZSubscriber,
    callback::{ZSubscriberCallback, ZSubscriberCallbackStorage},
};

pub(crate) mod publisher;
pub use publisher::ZPublisher;

pub(crate) mod replies;
pub use replies::callback::{ZRepliesCallback, ZRepliesCallbackStorage};

pub struct ZConfig<T: Platform + 'static, S1> {
    pub spawner: Spawner,
    pub platform: T,
    pub task: fn(driver: &'static SessionDriver<T>) -> SpawnToken<S1>,

    pub driver: &'static StaticCell<SessionDriver<T>>,
    pub transport: &'static StaticCell<Transport<T>>,

    pub tx_zbuf: &'static mut [u8],
    pub rx_zbuf: &'static mut [u8],

    pub subscribers: &'static mut dyn ZSubscriberCallbacks,
    pub queries: &'static mut dyn ZRepliesCallbacks,
}

#[macro_export]
macro_rules! zconfig {
    ($type:ident : ($spawner:expr, $platform:expr), TX: $TX:expr, RX: $RX:expr, MAX_SUBSCRIBERS: $MAX_SUBSCRIBERS:expr, MAX_QUERIES: $MAX_QUERIES:expr) => {{
        static DRIVER: static_cell::StaticCell<$crate::SessionDriver<$type>> =
            static_cell::StaticCell::new();

        static TRANSPORT: static_cell::StaticCell<$crate::Transport<$type>> =
            static_cell::StaticCell::new();

        static TX_ZBUF: static_cell::StaticCell<[u8; $TX]> = static_cell::StaticCell::new();
        static RX_ZBUF: static_cell::StaticCell<[u8; $RX]> = static_cell::StaticCell::new();

        static SUBSCRIBERS: static_cell::StaticCell<
            $crate::ZSubscriberCallbackStorage<$MAX_SUBSCRIBERS>,
        > = static_cell::StaticCell::new();

        static QUERIES: static_cell::StaticCell<$crate::ZRepliesCallbackStorage<$MAX_QUERIES>> =
            static_cell::StaticCell::new();

        #[embassy_executor::task]
        async fn session_task(runner: &'static $crate::SessionDriver<$type>) {
            if let Err(e) = runner.run().await {
                $crate::error!("Session driver task ended with error: {}", e);
            }
        }

        let zconfig = $crate::ZConfig {
            spawner: $spawner,
            platform: $platform,
            task: session_task,

            driver: &DRIVER,
            transport: &TRANSPORT,

            tx_zbuf: TX_ZBUF.init([0u8; $TX]).as_mut_slice(),
            rx_zbuf: RX_ZBUF.init([0u8; $RX]).as_mut_slice(),
            subscribers: SUBSCRIBERS
                .init($crate::ZSubscriberCallbackStorage::<$MAX_SUBSCRIBERS>::new()),
            queries: QUERIES.init($crate::ZRepliesCallbackStorage::<$MAX_QUERIES>::new()),
        };

        zconfig
    }};
}

#[macro_export]
macro_rules! open {
    ($zconfig:expr, $endpoint:expr) => {{
        let spawner = $zconfig.spawner.clone();
        let task = $zconfig.task.clone();
        let driver_cell = $zconfig.driver.clone();

        let (mut session, driver) = $crate::Session::new($zconfig, $endpoint).await?;

        let driver = driver_cell.init(driver);
        session.set_driver(driver);

        spawner
            .spawn((task)(driver))
            .map_err(|_| $crate::ZError::CouldNotSpawnTask)?;

        session
    }};
}
