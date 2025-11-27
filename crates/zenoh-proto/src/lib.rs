#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod logging;

mod error;
pub use error::*;

mod codec;
pub(crate) use codec::*;

mod core;
pub use core::*;

pub mod network;
pub mod transport;
pub mod zenoh;

mod batch;
pub use batch::*;

pub(crate) use zenoh_derive::*;

#[cfg(test)]
mod tests;
