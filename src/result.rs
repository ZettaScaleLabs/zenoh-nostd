#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZError {
    /// The channel is closed and no further communication is possible.
    ChannelClosed = 1,

    /// The channel is open but currently contains no data.
    ChannelNoData = 2,

    /// No data was processed during the operation.
    NoDataProcessed = 3,

    /// Failed to deserialize the received message.
    MessageDeserializationFailed = 119,

    /// Failed to serialize the message before sending.
    MessageSerializationFailed = 118,

    /// The message received was unexpected in this context.
    MessageUnexpected = 117,

    /// The message contained an unexpected flag.
    MessageFlagUnexpected = 116,

    /// The message referenced an unknown Zenoh declaration.
    MessageZenohDeclarationUnknown = 115,

    /// The message type is not recognized as a valid Zenoh message.
    MessageZenohUnknown = 114,

    /// The message uses an unknown transport format.
    MessageTransportUnknown = 113,

    /// The message contains a mandatory extension that is unknown.
    MessageExtensionMandatoryAndUnknown = 112,

    /// Failed to declare the specified entity.
    EntityDeclarationFailed = 111,

    /// The referenced entity is not recognized.
    EntityUnknown = 110,

    /// The key expression used is not recognized.
    KeyExprUnknown = 109,

    /// The key expression did not match any valid pattern.
    KeyExprNotMatch = 108,

    /// The query did not match any available data or expression.
    QueryNotMatch = 107,

    /// No suitable transport is available for the operation.
    TransportNotAvailable = 103,

    /// Failed to open the transport channel.
    TransportOpenFailed = 102,

    /// Failed to resolve service name when opening transport.
    TransportOpenSNResolution = 101,

    /// Transport failed during transmission.
    TransportTxFailed = 100,

    /// Transport failed during reception.
    TransportRxFailed = 99,

    /// The transport buffer has no available space.
    TransportNoSpace = 98,

    /// Transport did not receive enough bytes.
    TransportNotEnoughBytes = 97,

    /// Failed to insert value into configuration.
    ConfigFailedInsert = 95,

    /// The locator schema provided is unknown.
    ConfigLocatorSchemaUnknown = 92,

    /// The locator provided is invalid.
    ConfigLocatorInvalid = 91,

    /// The selected configuration mode is invalid.
    ConfigInvalidMode = 90,

    /// A generic system error occurred.
    SystemGeneric = 80,

    /// A system task failed unexpectedly.
    SystemTaskFailed = 79,

    /// The system ran out of memory.
    SystemOutOfMemory = 78,

    /// The connection was closed unexpectedly.
    ConnectionClosed = 77,

    /// No data was read during the operation.
    DidNotRead = 76,

    /// An invalid argument or value was provided.
    Invalid = 75,

    /// An overflow occurred during processing.
    Overflow = 74,

    /// No data was written during the operation.
    DidNotWrite = 70,

    /// The session has been closed.
    SessionClosed = 73,

    /// Failed to deserialize received data.
    Deserialize = 72,

    /// The operation timed out.
    TimedOut = 71,

    /// A generic, unspecified error occurred.
    Generic = 128,
}

impl core::fmt::Display for ZError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ZError::ChannelClosed => f.write_fmt(core::format_args!("channel closed")),
            ZError::ChannelNoData => f.write_fmt(core::format_args!("channel has no data")),
            ZError::NoDataProcessed => f.write_fmt(core::format_args!("no data processed")),
            ZError::MessageDeserializationFailed => {
                f.write_fmt(core::format_args!("message deserialization failed"))
            }
            ZError::MessageSerializationFailed => {
                f.write_fmt(core::format_args!("message serialization failed"))
            }
            ZError::MessageUnexpected => f.write_fmt(core::format_args!("unexpected message")),
            ZError::MessageFlagUnexpected => {
                f.write_fmt(core::format_args!("unexpected message flag"))
            }
            ZError::MessageZenohDeclarationUnknown => {
                f.write_fmt(core::format_args!("unknown zenoh declaration"))
            }
            ZError::MessageZenohUnknown => f.write_fmt(core::format_args!("unknown zenoh message")),
            ZError::MessageTransportUnknown => {
                f.write_fmt(core::format_args!("unknown transport message"))
            }
            ZError::MessageExtensionMandatoryAndUnknown => {
                f.write_fmt(core::format_args!("mandatory unknown extension"))
            }
            ZError::EntityDeclarationFailed => {
                f.write_fmt(core::format_args!("entity declaration failed"))
            }
            ZError::EntityUnknown => f.write_fmt(core::format_args!("unknown entity")),
            ZError::KeyExprUnknown => f.write_fmt(core::format_args!("unknown key expression")),
            ZError::KeyExprNotMatch => {
                f.write_fmt(core::format_args!("key expression not matched"))
            }
            ZError::QueryNotMatch => f.write_fmt(core::format_args!("query not matched")),
            ZError::TransportNotAvailable => {
                f.write_fmt(core::format_args!("transport not available"))
            }
            ZError::TransportOpenFailed => f.write_fmt(core::format_args!("transport open failed")),
            ZError::TransportOpenSNResolution => {
                f.write_fmt(core::format_args!("service name resolution failed"))
            }
            ZError::TransportTxFailed => {
                f.write_fmt(core::format_args!("transport transmit failed"))
            }
            ZError::TransportRxFailed => {
                f.write_fmt(core::format_args!("transport receive failed"))
            }
            ZError::TransportNoSpace => f.write_fmt(core::format_args!("no space in transport")),
            ZError::TransportNotEnoughBytes => {
                f.write_fmt(core::format_args!("not enough bytes received"))
            }
            ZError::ConfigFailedInsert => {
                f.write_fmt(core::format_args!("configuration insert failed"))
            }
            ZError::ConfigLocatorSchemaUnknown => {
                f.write_fmt(core::format_args!("unknown locator schema"))
            }
            ZError::ConfigLocatorInvalid => f.write_fmt(core::format_args!("invalid locator")),
            ZError::ConfigInvalidMode => {
                f.write_fmt(core::format_args!("invalid configuration mode"))
            }
            ZError::SystemGeneric => f.write_fmt(core::format_args!("system error")),
            ZError::SystemTaskFailed => f.write_fmt(core::format_args!("system task failed")),
            ZError::SystemOutOfMemory => f.write_fmt(core::format_args!("out of memory")),
            ZError::ConnectionClosed => f.write_fmt(core::format_args!("connection closed")),
            ZError::DidNotRead => f.write_fmt(core::format_args!("no data read")),
            ZError::Invalid => f.write_fmt(core::format_args!("invalid input")),
            ZError::Overflow => f.write_fmt(core::format_args!("overflow")),
            ZError::DidNotWrite => f.write_fmt(core::format_args!("no data written")),
            ZError::SessionClosed => f.write_fmt(core::format_args!("session closed")),
            ZError::Deserialize => f.write_fmt(core::format_args!("deserialization failed")),
            ZError::TimedOut => f.write_fmt(core::format_args!("operation timed out")),
            ZError::Generic => f.write_fmt(core::format_args!("generic error")),
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
