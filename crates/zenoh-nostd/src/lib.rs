#![cfg_attr(not(any(feature = "log", feature = "web_console")), no_std)]

pub mod api;
pub mod platform;

pub(crate) mod io;

pub use zenoh_proto::{EndPoint, debug, error, error::*, info, keyexpr, trace, warn, zbail};
