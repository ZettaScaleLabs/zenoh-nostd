#![cfg_attr(
    not(any(feature = "web_console", feature = "log", feature = "platform-std")),
    no_std
)]

pub mod logging;
pub mod result;

pub(crate) use zenoh_nostd_codec::*;

mod codec;
pub(crate) use codec::*;

mod protocol;
pub(crate) use protocol::*;
pub use protocol::{endpoint::EndPoint, ke::keyexpr};

pub(crate) mod platform;
#[cfg(feature = "platform-std")]
pub use platform::platform_std::PlatformStd;
pub use platform::{Platform, ZConnectionError, tcp::*};

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

        let (mut session, driver) = $crate::Session::new($zconfig, $endpoint).await?;

        let driver = driver_cell.init(driver);
        session.set_driver(driver);

        spawner
            .spawn((task)(driver))
            .map_err(|_| $crate::result::ZError::CouldNotSpawnTask)?;

        session
    }};
}
