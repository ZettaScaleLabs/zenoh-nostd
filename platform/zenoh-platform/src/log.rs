#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub use zenoh_platform_std::log;

#[cfg(target_arch = "wasm32")]
pub use zenoh_platform_wasm::log;

#[cfg(target_arch = "xtensa")]
pub use zenoh_platform_esp32::log;

pub fn init_logger() {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        zenoh_platform_std::log::init_logger();
    }
}
