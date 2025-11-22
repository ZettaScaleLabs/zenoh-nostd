use zenoh_proto::ZCodecError;

use crate::platform::ZConnectionError;

pub(crate) mod link;
pub(crate) mod transport;

zenoh_proto::make_zerr! {
    /// An error that can occur during link operations.
    #[err = "link error"]
    enum ZLinkError {
        CouldNotParse,
        CouldNotGetAddrInfo,
        CouldNotConnect,
        CouldNotWrite,
        CouldNotRead,
    }

    /// An error that can occur during transport operations.
    #[err = "transport error"]
    enum ZTransportError {
        InvalidRx,
        TxError,
        Timeout,

        CouldNotParse,
        CouldNotGetAddrInfo,
        CouldNotConnect,
        CouldNotWrite,
        CouldNotRead,
    }
}

impl From<ZConnectionError> for ZLinkError {
    fn from(e: ZConnectionError) -> Self {
        match e {
            ZConnectionError::CouldNotGetAddrInfo => ZLinkError::CouldNotGetAddrInfo,
            ZConnectionError::CouldNotConnect => ZLinkError::CouldNotConnect,
            ZConnectionError::CouldNotWrite => ZLinkError::CouldNotWrite,
            ZConnectionError::CouldNotRead => ZLinkError::CouldNotRead,
        }
    }
}

impl From<ZLinkError> for ZTransportError {
    fn from(e: ZLinkError) -> Self {
        match e {
            ZLinkError::CouldNotGetAddrInfo => ZTransportError::CouldNotGetAddrInfo,
            ZLinkError::CouldNotConnect => ZTransportError::CouldNotConnect,
            ZLinkError::CouldNotWrite => ZTransportError::CouldNotWrite,
            ZLinkError::CouldNotRead => ZTransportError::CouldNotRead,
            ZLinkError::CouldNotParse => ZTransportError::CouldNotParse,
        }
    }
}

impl From<ZCodecError> for ZTransportError {
    fn from(e: ZCodecError) -> Self {
        match e {
            ZCodecError::CouldNotRead => ZTransportError::InvalidRx,
            ZCodecError::CouldNotWrite => ZTransportError::TxError,
            ZCodecError::CouldNotParse => ZTransportError::InvalidRx,
            ZCodecError::MissingMandatoryExtension => ZTransportError::InvalidRx,
        }
    }
}
