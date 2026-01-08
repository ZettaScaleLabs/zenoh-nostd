#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod api;
pub use api::*;

mod io;
pub mod platform;

pub use zenoh_proto::{debug, error, info, logging, trace, warn, zbail, zerror::*};
