use wasm_bindgen::prelude::*;
use zenoh_platform::Platform;

pub struct PlatformWasm;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn js_log(s: &str);
}

impl Platform for PlatformWasm {
    type PlatformTcpStream = zenoh_platform::tcp::DummyPlatformTcpStream;

    fn init() -> zenoh_result::ZResult<Self> {
        Ok(Self {})
    }

    fn info(args: core::fmt::Arguments<'_>) {
        js_log(args.to_string().as_str());
    }

    fn warn(args: core::fmt::Arguments<'_>) {
        Self::info(args);
    }

    fn error(args: core::fmt::Arguments<'_>) {
        Self::info(args);
    }

    fn debug(args: core::fmt::Arguments<'_>) {
        Self::info(args);
    }

    fn trace(args: core::fmt::Arguments<'_>) {
        Self::info(args);
    }
}
