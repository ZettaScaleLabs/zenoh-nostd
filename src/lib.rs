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
    ($zconfig:expr, $endpoint:expr) => {{
        let spawner = $zconfig.spawner.clone();
        let task = $zconfig.task.clone();
        let driver_cell = $zconfig.driver.clone();

        let (mut session, driver) = $crate::api::session::Session::new($zconfig, $endpoint)
            .await
            .unwrap();

        let driver = driver_cell.init(driver);
        session.set_driver(driver);

        spawner.spawn((task)(driver)).unwrap();

        Ok::<_, $crate::result::ZError>(session)
    }};
}
