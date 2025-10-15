#![cfg_attr(
    not(any(feature = "web_console", feature = "log", feature = "platform-std")),
    no_std
)]

pub mod logging;
pub mod result;

pub(crate) mod zbuf;

pub(crate) mod keyexpr;
pub use keyexpr::borrowed::keyexpr as ke;

pub(crate) mod protocol;
pub use protocol::core::endpoint::EndPoint;

pub(crate) mod platform;
#[cfg(feature = "platform-std")]
pub use platform::platform_std::PlatformStd;
pub use platform::{Platform, tcp::*};

pub(crate) mod io;
pub use io::transport::Transport;

pub(crate) mod api;
pub use api::*;

#[cfg(test)]
pub(crate) mod tests;

/// This macro opens a new Zenoh session and spawns its driver task.
#[macro_export]
macro_rules! open {
    ($zconfig:expr, $endpoint:expr) => {{
        let spawner = $zconfig.spawner.clone();
        let task = $zconfig.task.clone();
        let driver_cell = $zconfig.driver.clone();

        let (mut session, driver) = $crate::Session::new($zconfig, $endpoint).await.unwrap();

        let driver = driver_cell.init(driver);
        session.set_driver(driver);

        spawner.spawn((task)(driver)).unwrap();

        Ok::<_, $crate::result::ZError>(session)
    }};
}
