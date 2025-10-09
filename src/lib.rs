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
    ($type:ident : ($spawner:expr, $platform:expr), $endpoint:expr) => {{
        let (mut session, driver) = $crate::api::session::Session::new($platform, $endpoint)
            .await
            .unwrap();

        static DRIVER: static_cell::StaticCell<$crate::api::session::SessionDriver<$type>> =
            static_cell::StaticCell::new();

        let driver: &'static $crate::api::session::SessionDriver<$type> = DRIVER.init(driver);
        session.set_driver(driver);

        #[embassy_executor::task]
        async fn session_task(runner: &'static $crate::api::session::SessionDriver<$type>) {
            runner.run().await;
        }

        $spawner.spawn(session_task(driver)).unwrap();

        Ok::<_, $crate::result::ZError>(session)
    }};
}
