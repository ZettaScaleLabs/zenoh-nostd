pub use defmt;

// #[macro_export]
// macro_rules! defmt_info {
//     ($($t:tt)*) => ($crate::log::defmt::info!("{}", format_args!($($t)*).to_string().as_str()));
// }

// #[macro_export]
// macro_rules! defmt_warn {
//     ($($t:tt)*) => ($crate::log::defmt::warn!("{}", format_args!($($t)*).to_string().as_str()));
// }

// #[macro_export]
// macro_rules! defmt_error {
//     ($($t:tt)*) => ($crate::log::defmt::error!("{}", format_args!($($t)*).to_string().as_str()));
// }

// #[macro_export]
// macro_rules! defmt_debug {
//     ($($t:tt)*) => ($crate::log::defmt::debug!("{}", format_args!($($t)*).to_string().as_str()));
// }

// #[macro_export]
// macro_rules! defmt_trace {
//     ($($t:tt)*) => ($crate::log::defmt::trace!("{}", format_args!($($t)*).to_string().as_str()));
// }

pub mod log {
    pub use defmt::debug;
    pub use defmt::error;
    pub use defmt::info;
    pub use defmt::trace;
    pub use defmt::warn;
}

pub fn init_logger() {}
