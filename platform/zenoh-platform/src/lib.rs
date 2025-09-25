#![cfg_attr(
    not(any(target_os = "linux", target_os = "macos", target_os = "windows",)),
    no_std
)]

pub mod log;
pub mod tcp;
