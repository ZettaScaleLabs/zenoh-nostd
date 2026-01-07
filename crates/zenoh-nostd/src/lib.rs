#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod api;
pub use api::ZResult;

pub mod platform;

pub(crate) mod io;

pub use zenoh_proto::{debug, error, info, logging, trace, warn, zbail, zerror::*};
