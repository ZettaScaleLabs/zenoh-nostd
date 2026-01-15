#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

// mod batch;
mod bytes;
mod codec;
mod ke;

pub const VERSION: u8 = 9;

pub mod logging;
pub mod msgs;
pub mod zerror;

// pub use batch::*;
pub(crate) use bytes::*;
pub(crate) use codec::*;
pub use codec::{
    decoder, encoder, encoder_ref, network_decoder, network_encoder, network_encoder_ref,
    transport_decoder, transport_encoder, transport_encoder_ref,
};
pub use ke::*;

pub use msgs::{exts, fields};
pub(crate) use zenoh_derive::*;
pub(crate) use zerror::*;

#[cfg(test)]
mod tests;

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZInstant(pub core::time::Duration);

impl From<core::time::Duration> for ZInstant {
    fn from(value: core::time::Duration) -> Self {
        Self(value)
    }
}

impl From<ZInstant> for core::time::Duration {
    fn from(value: ZInstant) -> Self {
        value.0
    }
}
