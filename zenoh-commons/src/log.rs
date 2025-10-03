#[cfg(feature = "log")]
pub use log;

#[cfg(feature = "log")]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        $crate::log::trace!($($arg)+)
    };
}

#[cfg(feature = "log")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::log::debug!($($arg)+)
    };
}

#[cfg(feature = "log")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::log::info!($($arg)+)
    };
}

#[cfg(feature = "log")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        $crate::log::warn!($($arg)+)
    };
}

#[cfg(feature = "log")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::log::error!($($arg)+)
    };
}

#[cfg(feature = "defmt")]
pub use defmt;

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        $crate::defmt::trace!($($arg)+)
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::defmt::debug!($($arg)+)
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::defmt::info!($($arg)+)
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        $crate::defmt::warn!($($arg)+)
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::defmt::error!($($arg)+)
    };
}

#[cfg(feature = "web_console")]
pub use web_sys::console;

#[cfg(feature = "web_console")]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        $crate::console::log_1(&format!($($arg)+).into())
    };
}

#[cfg(feature = "web_console")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::console::log_1(&format!($($arg)+).into())
    };
}

#[cfg(feature = "web_console")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::console::log_1(&format!($($arg)+).into())
    };
}

#[cfg(feature = "web_console")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        $crate::console::warn_1(&format!($($arg)+).into())
    };
}

#[cfg(feature = "web_console")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::console::error_1(&format!($($arg)+).into())
    };
}

#[cfg(not(any(feature = "log", feature = "defmt", feature = "web_console")))]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {};
}

#[cfg(not(any(feature = "log", feature = "defmt", feature = "web_console")))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {};
}

#[cfg(not(any(feature = "log", feature = "defmt", feature = "web_console")))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {};
}

#[cfg(not(any(feature = "log", feature = "defmt", feature = "web_console")))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {};
}

#[cfg(not(any(feature = "log", feature = "defmt", feature = "web_console")))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {};
}
