#[cfg(feature = "log")]
pub use log;

#[cfg(feature = "log")]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        $crate::logging::log::trace!($($arg)+)
    };
}

#[cfg(feature = "log")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::logging::log::debug!($($arg)+)
    };
}

#[cfg(feature = "log")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::logging::log::info!($($arg)+)
    };
}

#[cfg(feature = "log")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        $crate::logging::log::warn!($($arg)+)
    };
}

#[cfg(feature = "log")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::logging::log::error!($($arg)+)
    };
}

#[cfg(feature = "defmt")]
pub use defmt;

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {{
        use $crate::logging::defmt as defmt;
        defmt::trace!($($arg)+)
    }};
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {{
        use $crate::logging::defmt as defmt;
        defmt::debug!($($arg)+)
    }};
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {{
        use $crate::logging::defmt as defmt;
        defmt::info!($($arg)+)
    }};
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {{
        use $crate::logging::defmt as defmt;
        defmt::warn!($($arg)+)
    }};
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {{
        use $crate::logging::defmt as defmt;
        defmt::error!($($arg)+)
    }};
}

#[cfg(feature = "web_console")]
pub use web_sys::console;

#[cfg(feature = "web_console")]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        $crate::logging::console::trace_1(&format!($($arg)+).into())
    };
}

#[cfg(feature = "web_console")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::logging::console::debug_1(&format!($($arg)+).into())
    };
}

#[cfg(feature = "web_console")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::logging::console::info_1(&format!($($arg)+).into())
    };
}

#[cfg(feature = "web_console")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        $crate::logging::console::warn_1(&format!($($arg)+).into())
    };
}

#[cfg(feature = "web_console")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::logging::console::error_1(&format!($($arg)+).into())
    };
}

#[cfg(not(any(feature = "log", feature = "defmt", feature = "web_console")))]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {{
        let _ = format_args!($($arg)+);
    }};
}

#[cfg(not(any(feature = "log", feature = "defmt", feature = "web_console")))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {{
        let _ = format_args!($($arg)+);
    }};
}

#[cfg(not(any(feature = "log", feature = "defmt", feature = "web_console")))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {{
        let _ = format_args!($($arg)+);
    }};
}

#[cfg(not(any(feature = "log", feature = "defmt", feature = "web_console")))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {{
        let _ = format_args!($($arg)+);
    }};
}

#[cfg(not(any(feature = "log", feature = "defmt", feature = "web_console")))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {{
        let _ = format_args!($($arg)+);
    }};
}
