#![no_std]

pub mod common;
pub mod core;

pub mod zenoh;

pub mod network;
pub mod transport;

pub const VERSION: u8 = 0x09;
