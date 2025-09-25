pub use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn js_log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::log::js_log(&format_args!($($t)*).to_string()));
}

pub mod log {
    pub use console_log as info;
    pub use console_log as warn;
    pub use console_log as error;
    pub use console_log as debug;
    pub use console_log as trace;
}

pub fn init_logger() {}
