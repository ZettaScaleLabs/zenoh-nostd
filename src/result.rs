#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZError {
    /// a read operation could not be completed
    CouldNotRead = 1,

    /// a write operation could not be completed
    CouldNotWrite = 2,

    /// parsing a value failed
    CouldNotParse = 3,

    /// an argument provided to a function is invalid
    InvalidArgument = 4,

    /// a lone `$*` was found in an expression
    LoneDollarStar = 5,

    /// a single `*` was found after a `**` in an expression
    SingleStarAfterDoubleStar = 6,

    /// a double `**` was found after a `**` in an expression
    DoubleStarAfterDoubleStar = 7,

    /// an empty chunk was found in an expression
    EmptyChunk = 8,

    /// a `*` was found in the middle of a chunk in an expression
    StarInChunk = 9,

    /// a `$` was found after a `$` in an expression
    DollarAfterDollar = 10,

    /// a `#` or `?` was found in an expression
    SharpOrQMark = 11,

    /// an unbound `$n` was found in an expression
    UnboundDollar = 12,

    /// a wildcard chunk was found where it is not allowed
    WildChunk = 13,

    /// could not find protocol separator in endpoint
    NoProtocolSeparator = 14,

    /// metadata is not supported in endpoint
    MetadataNotSupported = 15,

    /// configuration is not supported in endpoint
    ConfigNotSupported = 16,

    /// could not connect to the specified address
    CouldNotConnect = 17,

    /// could not get address info
    CouldNotGetAddrInfo = 18,

    /// Received an invalid RX packet
    InvalidRx = 19,

    /// An error occurred while processing a TX packet
    TxError = 20,

    /// A timeout occurred
    Timeout = 21,

    /// The operation could not be completed because it would exceed some capacity limit
    CapacityExceeded = 22,

    /// Could not receive from subscriber
    CouldNotRecvFromSubscriber = 23,

    /// A subscriber callback is already set
    SubscriberCallbackAlreadySet = 24,

    /// The connection was closed
    ConnectionClosed = 25,
}

impl core::fmt::Display for ZError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ZError::CouldNotRead => f.write_str("Could not read"),
            ZError::CouldNotWrite => f.write_str("Could not write"),
            ZError::CouldNotParse => f.write_str("Could not parse"),
            ZError::InvalidArgument => f.write_str("Invalid argument"),
            ZError::LoneDollarStar => f.write_str("Lone '$*' in expression"),
            ZError::SingleStarAfterDoubleStar => f.write_str("Single '*' after '**' in expression"),
            ZError::DoubleStarAfterDoubleStar => {
                f.write_str("Double '**' after '**' in expression")
            }
            ZError::EmptyChunk => f.write_str("Empty chunk in expression"),
            ZError::StarInChunk => f.write_str("'*' in middle of chunk in expression"),
            ZError::DollarAfterDollar => f.write_str("'$' after '$' in expression"),
            ZError::SharpOrQMark => f.write_str("'#' or '?' in expression"),
            ZError::UnboundDollar => f.write_str("Unbound '$n' in expression"),
            ZError::WildChunk => f.write_str("Wildcard chunk where not allowed"),
            ZError::NoProtocolSeparator => {
                f.write_str("Could not find protocol separator in endpoint")
            }
            ZError::MetadataNotSupported => f.write_str("Metadata not supported in endpoint"),
            ZError::ConfigNotSupported => f.write_str("Configuration not supported in endpoint"),
            ZError::CouldNotConnect => f.write_str("Could not connect to the specified address"),
            ZError::CouldNotGetAddrInfo => f.write_str("Could not get address info"),
            ZError::InvalidRx => f.write_str("Received an invalid RX packet"),
            ZError::TxError => f.write_str("An error occurred while processing a TX packet"),
            ZError::Timeout => f.write_str("A timeout occurred"),
            ZError::CapacityExceeded => f.write_str("Capacity limit exceeded"),
            ZError::CouldNotRecvFromSubscriber => f.write_str("Could not receive from subscriber"),
            ZError::SubscriberCallbackAlreadySet => {
                f.write_str("A subscriber callback is already set")
            }
            ZError::ConnectionClosed => f.write_str("The connection was closed"),
        }
    }
}

#[macro_export]
macro_rules! __internal_zerr {
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
                    $variant = $crate::result::ZError::$variant as u8,
                )*
            }

            impl From<$name> for $crate::result::ZError {
                fn from(value: $name) -> Self {
                    match value {
                        $(
                            $name::$variant => $crate::result::ZError::$variant,
                        )*
                    }
                }
            }

            impl ::core::fmt::Display for $name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let zerror: $crate::result::ZError = (*self).into();
                    write!(f, "{}: {}", $err, zerror)
                }
            }

            #[cfg(feature = "defmt")]
            impl defmt::Format for $name {
                fn format(&self, f: defmt::Formatter) {
                    defmt::write!(f, "{}({}): {}", $err, *self as u8, self);
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
