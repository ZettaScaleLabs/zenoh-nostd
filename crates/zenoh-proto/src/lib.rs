#![cfg_attr(not(any(feature = "log", feature = "web_console")), no_std)]

mod logging;
#[cfg(any(feature = "log", feature = "defmt", feature = "web_console"))]
pub use logging::*;

mod error;
pub use error::*;

mod protocol;
pub use protocol::*;

pub(crate) use zenoh_derive::*;

#[cfg(test)]
mod tests;
