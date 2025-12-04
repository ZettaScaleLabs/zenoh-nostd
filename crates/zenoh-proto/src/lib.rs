#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub const VERSION: u8 = 9;

pub mod logging;

pub mod zerror;
pub(crate) use zerror::*;

mod bytes;
pub use bytes::*;

mod codec;
pub(crate) use codec::*;

mod ke;
pub use ke::*;

pub mod msgs;
pub use msgs::{exts, fields};

mod batch;
pub use batch::*;

pub(crate) use zenoh_derive::*;

#[cfg(test)]
mod tests;
