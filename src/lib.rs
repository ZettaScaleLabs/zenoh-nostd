#![cfg_attr(
    not(any(feature = "web_console", feature = "log", feature = "platform-std")),
    no_std
)]

pub mod logging;
pub mod result;
pub mod zbuf;

pub mod keyexpr;
pub mod protocol;

pub mod platform;

pub mod io;

pub mod api;

#[cfg(test)]
pub mod tests;

/// This macro opens a new Zenoh session and spawns its driver task.
/// The driver task is used to send the KeepAlive messages and maintain the session alive.
#[macro_export]
macro_rules! open {
    ($type:ident : ($spawner:expr, $platform:expr), TX: $TX:expr, RX: $RX:expr, SUBSCRIBERS: $SUBSCRIBERS:expr, $endpoint:expr) => {{
        static TX_ZBUF: static_cell::StaticCell<[u8; $TX]> = static_cell::StaticCell::new();
        static RX_ZBUF: static_cell::StaticCell<[u8; $RX]> = static_cell::StaticCell::new();

        static SUBSCRIBERS: static_cell::StaticCell<
            $crate::api::subscriber::ZSubscriberCallbackStorage<$SUBSCRIBERS>,
        > = static_cell::StaticCell::new();

        let zconfig = $crate::api::ZConfig {
            platform: $platform,
            tx_zbuf: TX_ZBUF.init([0u8; $TX]).as_mut_slice(),
            rx_zbuf: RX_ZBUF.init([0u8; $RX]).as_mut_slice(),
            subscribers: SUBSCRIBERS.init($crate::api::subscriber::ZSubscriberCallbackStorage::<
                $SUBSCRIBERS,
            >::new()),
        };

        let (mut session, driver) = $crate::api::session::Session::new(zconfig, $endpoint)
            .await
            .unwrap();

        static DRIVER: static_cell::StaticCell<$crate::api::driver::SessionDriver<$type>> =
            static_cell::StaticCell::new();

        let driver: &'static $crate::api::driver::SessionDriver<$type> = DRIVER.init(driver);
        session.set_driver(driver);

        #[embassy_executor::task]
        async fn session_task(runner: &'static $crate::api::driver::SessionDriver<$type>) {
            if let Err(e) = runner.run().await {
                $crate::error!("Session driver task ended with error: {}", e);
            }
        }

        $spawner.spawn(session_task(driver)).unwrap();

        Ok::<_, $crate::result::ZError>(session)
    }};
}
