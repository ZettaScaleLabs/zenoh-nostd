#![cfg_attr(not(any(feature = "log", feature = "web_console")), no_std)]

mod api;
pub use api::*;

pub(crate) mod io;
pub use io::transport::Transport;

pub mod platform;

pub use zenoh_proto::*;
