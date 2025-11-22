#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZError {
    // ──────────────── I/O errors (1–19) ────────────────
    /// Could not complete a read operation.
    CouldNotRead = 1,

    /// Could not complete a write operation.
    CouldNotWrite = 2,

    /// Could not parse the input.
    CouldNotParse = 3,

    /// Could not connect to the specified address.
    CouldNotConnect = 10,

    /// Could not get address information.
    CouldNotGetAddrInfo = 11,

    /// Timeout occurred.
    Timeout = 12,

    /// The operation would exceed a capacity limit.
    CapacityExceeded = 13,

    /// Received an invalid RX packet.
    InvalidRx = 14,

    /// An error occurred while processing a TX packet.
    TxError = 15,

    /// Could not receive from channel.
    CouldNotRecvFromChannel = 16,

    /// A subscriber callback is already set.
    CallbackAlreadySet = 17,

    /// The connection was closed.
    ConnectionClosed = 18,

    /// Could not spawn a task.
    CouldNotSpawnTask = 19,

    // Reserve: 19–29

    // ──────────────── Argument/validation errors (30–39) ────────────────
    /// An invalid argument was provided.
    InvalidArgument = 30,

    // Reserve: 31–39

    // ──────────────── Expression parsing errors (40–59) ────────────────
    /// A lone `$*` was found in an expression.
    LoneDollarStar = 40,

    /// A single `*` was found after a `**` in an expression.
    SingleStarAfterDoubleStar = 41,

    /// A double `**` was found after a `**` in an expression.
    DoubleStarAfterDoubleStar = 42,

    /// An empty chunk was found in an expression.
    EmptyChunk = 43,

    /// A `*` was found in the middle of a chunk in an expression.
    StarInChunk = 44,

    /// A `$` was found after another `$` in an expression.
    DollarAfterDollar = 45,

    /// A `#` or `?` was found in an expression.
    SharpOrQMark = 46,

    /// An unbound `$n` was found in an expression.
    UnboundDollar = 47,

    /// A wildcard chunk was found where it is not allowed.
    WildChunk = 48,

    // Reserve: 49–59

    // ──────────────── Endpoint errors (60–69) ────────────────
    /// Missing protocol separator in endpoint.
    NoProtocolSeparator = 60,

    /// Metadata is not supported in endpoint.
    MetadataNotSupported = 61,

    /// Configuration is not supported in endpoint.
    ConfigNotSupported = 62,
    // Reserve: 63–69

    // ──────────────── Protocol errors (70–255) ────────────────
    MissingMandatoryExtension = 70,
}

impl core::fmt::Display for ZError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            // I/O errors
            ZError::CouldNotRead => f.write_str("could not read"),
            ZError::CouldNotWrite => f.write_str("could not write"),
            ZError::CouldNotParse => f.write_str("could not parse"),
            ZError::CouldNotConnect => f.write_str("could not connect to the specified address"),
            ZError::CouldNotGetAddrInfo => f.write_str("could not get address info"),
            ZError::Timeout => f.write_str("timeout occurred"),
            ZError::CapacityExceeded => f.write_str("capacity limit exceeded"),
            ZError::InvalidRx => f.write_str("invalid rx packet"),
            ZError::TxError => f.write_str("tx error occurred"),
            ZError::CouldNotRecvFromChannel => f.write_str("could not receive from channel"),
            ZError::CallbackAlreadySet => f.write_str("callback already set"),
            ZError::ConnectionClosed => f.write_str("connection closed"),
            ZError::CouldNotSpawnTask => f.write_str("could not spawn task"),

            // Argument errors
            ZError::InvalidArgument => f.write_str("invalid argument"),

            // Expression parsing errors
            ZError::LoneDollarStar => f.write_str("lone '$*' in expression"),
            ZError::SingleStarAfterDoubleStar => f.write_str("single '*' after '**' in expression"),
            ZError::DoubleStarAfterDoubleStar => {
                f.write_str("double '**' after '**' in expression")
            }
            ZError::EmptyChunk => f.write_str("empty chunk in expression"),
            ZError::StarInChunk => f.write_str("'*' in middle of chunk in expression"),
            ZError::DollarAfterDollar => f.write_str("'$' after '$' in expression"),
            ZError::SharpOrQMark => f.write_str("'#' or '?' in expression"),
            ZError::UnboundDollar => f.write_str("unbound '$n' in expression"),
            ZError::WildChunk => f.write_str("wildcard chunk not allowed"),

            // Endpoint errors
            ZError::NoProtocolSeparator => f.write_str("missing protocol separator in endpoint"),
            ZError::MetadataNotSupported => f.write_str("metadata not supported in endpoint"),
            ZError::ConfigNotSupported => f.write_str("configuration not supported in endpoint"),

            // Protocol errors
            ZError::MissingMandatoryExtension => f.write_str("missing mandatory extension"),
        }
    }
}

#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! make_zerr {
    (
        $(
            #[doc = $doc:literal]
            #[err = $err:literal]
            enum $name:ident {
                $(
                    $variant:ident
                ),* $(,)?
            }
        )*
    ) => {
        $(
            #[allow(clippy::enum_variant_names)]
            #[repr(u8)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            #[doc = $doc]
            pub enum $name {
                $(
                    #[doc = concat!("See [`ZError::", stringify!($variant), "`]")]
                    $variant = $crate::ZError::$variant as u8,
                )*
            }

            impl From<$name> for $crate::ZError {
                fn from(value: $name) -> Self {
                    match value {
                        $(
                            $name::$variant => $crate::ZError::$variant,
                        )*
                    }
                }
            }

            impl ::core::fmt::Display for $name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let zerror: $crate::ZError = (*self).into();
                    write!(f, "{}: {}", $err, zerror)
                }
            }
        )*
    };
}

#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! make_zerr {
    (
        $(
            #[doc = $doc:literal]
            #[err = $err:literal]
            enum $name:ident {
                $(
                    $variant:ident
                ),* $(,)?
            }
        )*
    ) => {
        $(
            #[allow(clippy::enum_variant_names)]
            #[repr(u8)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            #[doc = $doc]
            pub enum $name {
                $(
                    #[doc = concat!("See [`ZError::", stringify!($variant), "`]")]
                    $variant = $crate::ZError::$variant as u8,
                )*
            }

            impl From<$name> for $crate::ZError {
                fn from(value: $name) -> Self {
                    match value {
                        $(
                            $name::$variant => $crate::ZError::$variant,
                        )*
                    }
                }
            }

            impl ::core::fmt::Display for $name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let zerror: $crate::ZError = (*self).into();
                    write!(f, "{}: {}", $err, zerror)
                }
            }

            impl $crate::defmt::Format for $name {
                fn format(&self, f: $crate::defmt::Formatter) {
                    use $crate::defmt;
                    $crate::defmt::write!(f, "{}({}): {}", $err, *self as u8, self);
                }
            }
        )*
    };
}

pub type ZResult<T, E = ZError> = core::result::Result<T, E>;

#[macro_export]
macro_rules! zbail {
    ($err:expr) => {
        return Err($err)
    };
}

#[cfg(feature = "defmt")]
impl defmt::Format for ZError {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "ZError({}): {}", *self as u8, self);
    }
}
