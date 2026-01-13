#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod api;
mod io;

pub use api::*;
pub mod platform;
pub use zenoh_proto::{debug, error, info, logging, trace, warn, zbail, zctx, zerror::*};
