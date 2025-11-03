#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
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

    /// Could not receive from subscriber.
    CouldNotRecvFromSubscriber = 16,

    /// A subscriber callback is already set.
    SubscriberCallbackAlreadySet = 17,

    /// The connection was closed.
    ConnectionClosed = 18,

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
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            // I/O errors
            Error::CouldNotRead => f.write_str("could not read"),
            Error::CouldNotWrite => f.write_str("could not write"),
            Error::CouldNotParse => f.write_str("could not parse"),
            Error::CouldNotConnect => f.write_str("could not connect to the specified address"),
            Error::CouldNotGetAddrInfo => f.write_str("could not get address info"),
            Error::Timeout => f.write_str("timeout occurred"),
            Error::CapacityExceeded => f.write_str("capacity limit exceeded"),
            Error::InvalidRx => f.write_str("invalid rx packet"),
            Error::TxError => f.write_str("tx error occurred"),
            Error::CouldNotRecvFromSubscriber => f.write_str("could not receive from subscriber"),
            Error::SubscriberCallbackAlreadySet => f.write_str("subscriber callback already set"),
            Error::ConnectionClosed => f.write_str("connection closed"),

            // Argument errors
            Error::InvalidArgument => f.write_str("invalid argument"),

            // Expression parsing errors
            Error::LoneDollarStar => f.write_str("lone '$*' in expression"),
            Error::SingleStarAfterDoubleStar => f.write_str("single '*' after '**' in expression"),
            Error::DoubleStarAfterDoubleStar => f.write_str("double '**' after '**' in expression"),
            Error::EmptyChunk => f.write_str("empty chunk in expression"),
            Error::StarInChunk => f.write_str("'*' in middle of chunk in expression"),
            Error::DollarAfterDollar => f.write_str("'$' after '$' in expression"),
            Error::SharpOrQMark => f.write_str("'#' or '?' in expression"),
            Error::UnboundDollar => f.write_str("unbound '$n' in expression"),
            Error::WildChunk => f.write_str("wildcard chunk not allowed"),

            // Endpoint errors
            Error::NoProtocolSeparator => f.write_str("missing protocol separator in endpoint"),
            Error::MetadataNotSupported => f.write_str("metadata not supported in endpoint"),
            Error::ConfigNotSupported => f.write_str("configuration not supported in endpoint"),
        }
    }
}

#[macro_export]
macro_rules! __internal_err {
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
                    #[doc = concat!("See [`Error::", stringify!($variant), "`]")]
                    $variant = $crate::result::Error::$variant as u8,
                )*
            }

            impl From<$name> for $crate::result::Error {
                fn from(value: $name) -> Self {
                    match value {
                        $(
                            $name::$variant => $crate::result::Error::$variant,
                        )*
                    }
                }
            }

            impl ::core::fmt::Display for $name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let zerror: $crate::result::Error = (*self).into();
                    write!(f, "{}: {}", $err, zerror)
                }
            }
        )*
    };
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err)
    };
}
