#![cfg_attr(not(any(feature = "log", feature = "web_console")), no_std)]

mod session;
pub use session::*;
