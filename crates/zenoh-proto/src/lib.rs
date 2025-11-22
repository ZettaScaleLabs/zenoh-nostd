#![cfg_attr(not(any(feature = "log")), no_std)]

mod logging;
#[cfg(any(feature = "log", feature = "defmt"))]
pub use logging::*;

mod error;
pub use error::*;

mod protocol;
pub use protocol::*;

pub(crate) use zenoh_derive::*;

#[cfg(test)]
mod tests;
