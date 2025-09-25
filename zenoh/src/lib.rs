#![cfg_attr(
    not(any(target_os = "linux", target_os = "macos", target_os = "windows",)),
    no_std
)]
pub use zenoh_platform::log::{init_logger, log::*};

pub use zenoh_protocol::core::key_expr::keyexpr;
pub use zenoh_protocol::core::EndPoint;

pub mod api;
